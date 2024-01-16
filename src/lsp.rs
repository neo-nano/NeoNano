use anyhow::anyhow;
use core::time::Duration;
use lsp_types::{
    lsp_notification, lsp_request, ClientCapabilities, DidOpenTextDocumentParams, Hover,
    HoverClientCapabilities, HoverParams, InitializeParams, InitializedParams, MarkupKind,
    Position, TextDocumentClientCapabilities, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TextDocumentSyncClientCapabilities, Url,
    WorkspaceClientCapabilities,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::thread::sleep;

static JSON_RPC: &str = "2.0";

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Request {
    jsonrpc: &'static str,
    #[serde(default)]
    method: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Response<'a> {
    jsonrpc: &'a str,
    result: Option<Value>,
    error: Option<Value>,
}

pub struct LspConnector {
    initialized: bool,
    tx: Sender<String>,
    rx: Receiver<String>,
    child: Child,
    lang: String,
    filename: String,
}

impl LspConnector {
    fn start_process(
        sender: Sender<String>,
        receiver: Receiver<String>,
        path: &str,
        args: Vec<&str>,
    ) -> anyhow::Result<Child> {
        fn start_process_thread(
            child: &mut Child,
            sender: Sender<String>,
            receiver: Receiver<String>,
        ) {
            let mut stdin = child.stdin.take().unwrap();
            let stdout = child.stdout.take().unwrap();

            thread::spawn(move || {
                let mut buf: String = String::new();
                let mut f = BufReader::new(stdout);

                loop {
                    buf.truncate(0);
                    match f.read_line(&mut buf) {
                        Ok(_) => {
                            if !buf.to_lowercase().starts_with("content-length: ") {
                                continue;
                            }
                            let len_str = buf.get(16..).unwrap().strip_suffix("\r\n").unwrap();
                            let len: usize = len_str.parse().unwrap();
                            let mut content: Vec<u8> = vec![0; len];
                            f.consume("\r\n".len());
                            match f.read_exact(content.as_mut_slice()) {
                                Ok(_) => {
                                    let s = String::from_utf8(content).unwrap();
                                    sender.send(s).unwrap();
                                }
                                Err(e) => {
                                    println!("an error!: {:?}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            println!("an error!: {:?}", e);
                            break;
                        }
                    }
                }
            });

            thread::spawn(move || loop {
                match receiver.recv() {
                    Ok(line) => {
                        stdin.write_all(line.as_bytes()).unwrap();
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                }
            });
        }
        let mut child = Command::new(path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        start_process_thread(&mut child, sender, receiver);
        Ok(child)
    }
    pub fn new(
        lsp_path: &str,
        lsp_args: Vec<&str>,
        lang: String,
        filename: String,
    ) -> anyhow::Result<Self> {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let child = Self::start_process(tx1, rx2, lsp_path, lsp_args)?;
        Ok(Self {
            initialized: false,
            tx: tx2,
            rx: rx1,
            child,
            filename,
            lang,
        })
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn init(&mut self, current_text: String) {
        let init = Request::from_request::<lsp_request!("initialize")>(
            0,
            InitializeParams {
                process_id: None,
                root_path: None,
                root_uri: None,
                initialization_options: None,
                capabilities: ClientCapabilities {
                    workspace: Some(WorkspaceClientCapabilities {
                        apply_edit: None,
                        workspace_edit: None,
                        did_change_configuration: None,
                        did_change_watched_files: None,
                        symbol: None,
                        execute_command: None,
                        workspace_folders: Some(true),
                        configuration: None,
                        semantic_tokens: None,
                        code_lens: None,
                        file_operations: None,
                        inline_value: None,
                        inlay_hint: None,
                        diagnostic: None,
                    }),
                    text_document: Some(TextDocumentClientCapabilities {
                        synchronization: Some(TextDocumentSyncClientCapabilities {
                            dynamic_registration: Some(true),
                            will_save: None,
                            will_save_wait_until: None,
                            did_save: None,
                        }),
                        completion: None,
                        hover: Some(HoverClientCapabilities {
                            dynamic_registration: Some(true),
                            content_format: Some(vec![MarkupKind::PlainText]),
                        }),
                        signature_help: None,
                        references: None,
                        document_highlight: None,
                        document_symbol: None,
                        formatting: None,
                        range_formatting: None,
                        on_type_formatting: None,
                        declaration: None,
                        definition: None,
                        type_definition: None,
                        implementation: None,
                        code_action: None,
                        code_lens: None,
                        document_link: None,
                        color_provider: None,
                        rename: None,
                        publish_diagnostics: None,
                        folding_range: None,
                        selection_range: None,
                        linked_editing_range: None,
                        call_hierarchy: None,
                        semantic_tokens: None,
                        moniker: None,
                        type_hierarchy: None,
                        inline_value: None,
                        inlay_hint: None,
                        diagnostic: None,
                    }),
                    window: None,
                    general: None,
                    experimental: None,
                },
                trace: None,
                workspace_folders: None,
                client_info: None,
                locale: None,
                work_done_progress_params: Default::default(),
            },
        );
        self.send_request(&init);
        self.recv().unwrap();

        let init_notify =
            Request::from_notification::<lsp_notification!("initialized")>(InitializedParams {});
        self.send_request(&init_notify);

        let open_notify = Request::from_notification::<lsp_notification!("textDocument/didOpen")>(
            DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: Url::try_from(format!("file:///{}", self.filename).as_str()).unwrap(),
                    language_id: self.lang.clone(),
                    version: 0,
                    text: current_text,
                },
            },
        );
        self.send_request(&open_notify);

        self.initialized = true;
    }

    pub fn hover(&self, line: u32, character: u32) -> Option<Hover> {
        let hover = Request::from_request::<lsp_request!("textDocument/hover")>(
            1,
            HoverParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier {
                        uri: Url::try_from(format!("file:///{}", self.filename).as_str()).unwrap(),
                    },
                    position: Position { line, character },
                },
                work_done_progress_params: Default::default(),
            },
        );

        self.send_request(&hover);
        let res = self.recv().unwrap_or_default();
        if let Ok(res) = serde_json::from_str::<Response>(res.as_str()) {
            if let Some(params) = res.result {
                if let Ok(hover) = serde_json::from_value::<Hover>(params) {
                    return Some(hover);
                }
            }
        }
        None
    }

    fn send_request(&self, req: &Request) {
        let s = serde_json::to_string(req).unwrap();
        let payload = format!("Content-Length: {}\r\n\r\n{}", s.len(), s);
        self.tx.send(payload).unwrap();
    }

    fn try_recv(&self) -> Option<String> {
        match self.rx.try_recv() {
            Ok(line) => Some(line),
            Err(_) => None,
        }
    }

    fn recv(&self) -> anyhow::Result<String> {
        loop {
            match self.rx.try_recv() {
                Ok(line) => {
                    return Ok(line);
                }
                Err(TryRecvError::Empty) => {
                    sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!("Failed to receive LSP response: {e}"));
                }
            }
        }
    }
}

impl Request {
    fn from_request<R>(id: i32, params: R::Params) -> Self
    where
        R: lsp_types::request::Request,
    {
        Request {
            jsonrpc: JSON_RPC,
            method: R::METHOD.into(),
            params: Some(serde_json::to_value(params).unwrap()),
            id: Some(id),
        }
    }

    fn from_notification<R>(params: R::Params) -> Self
    where
        R: lsp_types::notification::Notification,
    {
        Request {
            jsonrpc: JSON_RPC,
            method: R::METHOD.into(),
            params: Some(serde_json::to_value(params).unwrap()),
            id: None,
        }
    }
}
