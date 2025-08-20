use crate::imports::*;
use tondi_rpc_core::RpcPeerInfo;

pub const PEER_POLLING_INTERVAL_SECONDS: u64 = 1; // 1 sec

pub enum PeerMonitorEvents {
    Enable,
    Disable,
    Exit,
}

pub struct PeerMonitorService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<PeerMonitorEvents>,
    pub task_ctl: Channel<()>,
    pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    pub peer_info: Mutex<Option<Arc<Vec<RpcPeerInfo>>>>,
    pub is_enabled: Arc<AtomicBool>,
}

impl PeerMonitorService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            rpc_api: Mutex::new(None),
            peer_info: Mutex::new(None),
            is_enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn rpc_api(&self) -> Option<Arc<dyn RpcApi>> {
        self.rpc_api.lock().unwrap().clone()
    }

    pub fn peer_info(&self) -> Option<Arc<Vec<RpcPeerInfo>>> {
        self.peer_info.lock().unwrap().clone()
    }

    pub fn enable(&self) {
        self.service_events
            .sender
            .try_send(PeerMonitorEvents::Enable)
            .unwrap();
    }

    pub fn disable(&self) {
        self.service_events
            .sender
            .try_send(PeerMonitorEvents::Disable)
            .unwrap();
    }
}

#[async_trait]
impl Service for PeerMonitorService {
    fn name(&self) -> &'static str {
        "peer-monitor"
    }

    async fn attach_rpc(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        self.rpc_api.lock().unwrap().replace(rpc_api.clone());
        Ok(())
    }

    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.rpc_api.lock().unwrap().take();
        self.peer_info.lock().unwrap().take();

        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let _application_events_sender = self.application_events.sender.clone();

        let interval = task::interval(Duration::from_secs(PEER_POLLING_INTERVAL_SECONDS));
        pin_mut!(interval);

        loop {
            select! {
                _ = interval.next().fuse() => {
                    if !self.is_enabled.load(Ordering::Relaxed) {
                        continue;
                    }

                    println!("[PeerMonitor] Polling for peer information...");
                    
                    if let Some(rpc_api) = this.rpc_api() {
                        println!("[PeerMonitor] RPC API available, requesting peer info...");
                        
                        match rpc_api.get_connected_peer_info().await {
                            Ok(resp) => {
                                println!("[PeerMonitor] Successfully received peer info: {} peers", resp.peer_info.len());
                                this.peer_info.lock().unwrap().replace(Arc::new(resp.peer_info));
                                
                                // 发送事件通知UI更新
                                if let Err(e) = _application_events_sender.try_send(crate::events::Events::UpdateLogs) {
                                    println!("[PeerMonitor] Failed to send update logs event: {}", e);
                                }
                            }
                            Err(e) => {
                                println!("[PeerMonitor] Failed to get peer info: {}", e);
                            }
                        }
                    } else {
                        println!("[PeerMonitor] No RPC API available");
                    }
                },
                msg = this.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            PeerMonitorEvents::Enable => {
                                self.is_enabled.store(true, Ordering::Relaxed);
                            }
                            PeerMonitorEvents::Disable => {
                                self.is_enabled.store(false, Ordering::Relaxed);
                                this.peer_info.lock().unwrap().take();
                            }
                            PeerMonitorEvents::Exit => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        this.task_ctl.send(()).await.unwrap();
        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(PeerMonitorEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
