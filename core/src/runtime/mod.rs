use crate::imports::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub mod signals;
        pub mod panic;
    } else {
        // ...
    }
}

pub mod channel;
pub mod payload;
pub mod services;
pub mod system;
// use crate::adaptor::{Adaptor, WebEvent};
// use crate::adaptor::{Adaptor, WebEvent};
use crate::interop::Adaptor;
pub use payload::Payload;
pub use services::Service;
use services::*;
use system::*;

pub struct Inner {
    services: Mutex<Vec<Arc<dyn Service>>>,
    repaint_service: Arc<RepaintService>,
    application_events: ApplicationEventsChannel,
    egui_ctx: egui::Context,
    is_running: Arc<AtomicBool>,
    start_time: Instant,
    system: Option<System>,

    // adaptor: Option<Arc<dyn Adaptor>>,
    adaptor: Option<Arc<Adaptor>>,

    tondi: Arc<TondiService>,
    peer_monitor_service: Arc<PeerMonitorService>,
    feerate_monitor_service: Arc<FeerateMonitorService>,
    update_monitor_service: Arc<UpdateMonitorService>,
    market_monitor_service: Arc<MarketMonitorService>,

    // #[cfg(not(feature = "lean"))]
    metrics_service: Arc<MetricsService>,
    #[cfg(not(feature = "lean"))]
    block_dag_monitor_service: Arc<BlockDagMonitorService>,
}

/// Runtime is a core component of the Tondi Dashboard application responsible for
/// running application services and communication between these services
/// and the application UI.
#[derive(Clone)]
pub struct Runtime {
    inner: Arc<Inner>,
}

impl Runtime {
    pub fn new(
        egui_ctx: &egui::Context,
        settings: &Settings,
        wallet_api: Option<Arc<dyn WalletApi>>,
        application_events: Option<ApplicationEventsChannel>,
        // adaptor: Option<Arc<dyn Adaptor>>,
        adaptor: Option<Arc<Adaptor>>,
    ) -> Self {
        let system = System::new();

        let application_events =
            application_events.unwrap_or_else(ApplicationEventsChannel::unbounded);
        let repaint_service = Arc::new(RepaintService::new(application_events.clone(), settings));
        let tondi = Arc::new(TondiService::new(
            application_events.clone(),
            settings,
            wallet_api,
        ));
        let peer_monitor_service = Arc::new(PeerMonitorService::new(
            application_events.clone(),
            settings,
        ));
        let feerate_monitor_service = Arc::new(FeerateMonitorService::new(
            application_events.clone(),
            settings,
        ));
        let market_monitor_service = Arc::new(MarketMonitorService::new(
            application_events.clone(),
            settings,
        ));

        let update_monitor_service = Arc::new(UpdateMonitorService::new(
            application_events.clone(),
            settings,
        ));

        let metrics_service = Arc::new(MetricsService::new(application_events.clone(), settings));
        cfg_if! {
            if #[cfg(not(feature = "lean"))] {
                let block_dag_monitor_service = Arc::new(BlockDagMonitorService::new(
                    application_events.clone(),
                    settings,
                ));
            }
        }
        // let metrics_service = Arc::new(MetricsService::new(application_events.clone(), settings));
        // let block_dag_monitor_service = Arc::new(BlockDagMonitorService::new(
        //     application_events.clone(),
        //     settings,
        // ));

        let services: Mutex<Vec<Arc<dyn Service>>> = Mutex::new(vec![
            repaint_service.clone(),
            tondi.clone(),
            peer_monitor_service.clone(),
            feerate_monitor_service.clone(),
            market_monitor_service.clone(),
            update_monitor_service.clone(),
            // #[cfg(not(feature = "lean"))]
            metrics_service.clone(),
            #[cfg(not(feature = "lean"))]
            block_dag_monitor_service.clone(),
        ]);

        let runtime = Self {
            inner: Arc::new(Inner {
                services,
                application_events,
                repaint_service,
                tondi,
                feerate_monitor_service,
                peer_monitor_service,
                market_monitor_service,
                update_monitor_service,
                egui_ctx: egui_ctx.clone(),
                is_running: Arc::new(AtomicBool::new(false)),
                start_time: Instant::now(),
                system: Some(system),
                adaptor,
                // #[cfg(not(feature = "lean"))]
                metrics_service,
                #[cfg(not(feature = "lean"))]
                block_dag_monitor_service,
            }),
        };

        register_global(Some(runtime.clone()));

        runtime
    }

    // pub fn set_adaptor(&self, adapter:Adaptor){
    //     self.inner.adaptor.lock().unwrap().adaptor = Some(adapter);
    // }

    // pub fn post_to_server(&self, event: WebEvent) {
    //     if let Some(adaptor) = self.inner.adaptor.as_ref() {
    //         adaptor.post_to_server(event);
    //     }
    // }

    pub fn adaptor(&self) -> &Option<Arc<Adaptor>> {
        &self.inner.adaptor
    }

    pub fn uptime(&self) -> Duration {
        self.inner.start_time.elapsed()
    }

    pub fn system(&self) -> &Option<System> {
        &self.inner.system
    }

    pub fn start_services(&self) {
        let services = self.services();
        for service in services {
            spawn(async move { service.spawn().await });
        }
    }

    pub fn services(&self) -> Vec<Arc<dyn Service>> {
        self.inner.services.lock().unwrap().clone()
    }

    pub fn stop_services(&self) {
        self.services()
            .into_iter()
            .for_each(|service| service.terminate());
    }

    pub async fn join_services(&self) {
        // for service in self.services() {
        //  let name = service.name();
        //  println!("⚡ {name}");
        //  service.join().await.expect("service join failure");
        //  println!("💀 {name}");
        // }

        let futures = self
            .services()
            .into_iter()
            .map(|service| service.join())
            .collect::<Vec<_>>();
        join_all(futures).await;
    }

    pub fn drop(&self) {
        register_global(None);
    }

    // / Start the runtime runtime.
    pub fn start(&self) {
        self.inner.is_running.store(true, Ordering::SeqCst);
        self.start_services();
    }

    /// Shutdown runtime runtime.
    pub async fn shutdown(&self) {
        if self.inner.is_running.load(Ordering::SeqCst) {
            self.inner.is_running.store(false, Ordering::SeqCst);
            self.stop_services();
            
            // 添加超时机制，防止服务关闭时卡住
            let timeout = tokio::time::Duration::from_secs(10);
            match tokio::time::timeout(timeout, self.join_services()).await {
                Ok(_) => println!("All services shut down successfully"),
                Err(_) => println!("Warning: Some services took too long to shut down, forcing shutdown"),
            }
            
            register_global(None);
        }
    }

    /// Returns the reference to the wallet API.
    pub fn wallet(&self) -> Arc<dyn WalletApi> {
        self.inner.tondi.wallet()
    }

    pub fn repaint_service(&self) -> &Arc<RepaintService> {
        &self.inner.repaint_service
    }

    /// Returns the reference to the tondi service.
    pub fn tondi_service(&self) -> &Arc<TondiService> {
        &self.inner.tondi
    }

    pub fn feerate_monitor_service(&self) -> &Arc<FeerateMonitorService> {
        &self.inner.feerate_monitor_service
    }

    pub fn peer_monitor_service(&self) -> &Arc<PeerMonitorService> {
        &self.inner.peer_monitor_service
    }

    pub fn metrics_service(&self) -> &Arc<MetricsService> {
        &self.inner.metrics_service
    }

    cfg_if! {
        if #[cfg(not(feature = "lean"))] {

            pub fn block_dag_monitor_service(&self) -> &Arc<BlockDagMonitorService> {
                &self.inner.block_dag_monitor_service
            }
        }
    }

    pub fn market_monitor_service(&self) -> &Arc<MarketMonitorService> {
        &self.inner.market_monitor_service
    }

    pub fn update_monitor_service(&self) -> &Arc<UpdateMonitorService> {
        &self.inner.update_monitor_service
    }

    /// Returns the reference to the application events channel.
    pub fn application_events(&self) -> &ApplicationEventsChannel {
        &self.inner.application_events
    }

    /// Send an application even to the UI asynchronously.
    pub async fn send(&self, msg: Events) -> Result<()> {
        self.inner.application_events.sender.send(msg).await?;
        Ok(())
    }

    /// Send an application event to the UI synchronously.
    pub fn try_send(&self, msg: Events) -> Result<()> {
        self.inner.application_events.sender.try_send(msg)?;
        Ok(())
    }

    /// Update storage size
    pub fn update_storage(&self, options: StorageUpdateOptions) {
        self.inner
            .application_events
            .sender
            .try_send(Events::UpdateStorage(options))
            .ok();
    }

    pub fn notify(&self, user_notification: UserNotification) {
        self.inner
            .application_events
            .sender
            .try_send(Events::Notify { user_notification })
            .ok();
    }

    pub fn error(&self, text: impl Into<String>) {
        self.inner
            .application_events
            .sender
            .try_send(Events::Notify {
                user_notification: UserNotification::error(text),
            })
            .ok();
    }

    pub fn toast(&self, user_notification: UserNotification) {
        self.inner
            .application_events
            .sender
            .try_send(Events::Notify {
                user_notification: user_notification.as_toast(),
            })
            .ok();
    }

    pub fn notify_clipboard(&self, text: impl Into<String>) {
        use egui_phosphor::light::CLIPBOARD_TEXT;
        let user_notification = UserNotification::info(format!("{CLIPBOARD_TEXT} {}", text.into()))
            .short()
            .as_toast();

        self.inner
            .application_events
            .sender
            .try_send(Events::Notify { user_notification })
            .ok();
    }

    pub fn spawn_task<F>(&self, task: F)
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let sender = self.inner.application_events.sender.clone();
        workflow_core::task::spawn(async move {
            if let Err(err) = task.await {
                sender
                    .send(Events::Error(Box::new(err.to_string())))
                    .await
                    .unwrap();
            }
        });
    }

    pub fn spawn_task_with_result<R, F>(
        &self,
        payload: &Payload<std::result::Result<R, Error>>,
        task: F,
    ) where
        R: Clone + Send + 'static,
        F: Future<Output = Result<R>> + Send + 'static,
    {
        let payload = (*payload).clone();
        payload.mark_pending();
        workflow_core::task::spawn(async move {
            let result = task.await;
            match result {
                Ok(r) => payload.store(Ok(r)),
                Err(err) => {
                    payload.store(Err(err));
                }
            }
        });
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.inner.egui_ctx
    }

    pub fn request_repaint(&self) {
        self.repaint_service().smart_trigger();
    }

    /// 强制立即重绘（用于重要更新）
    pub fn force_repaint(&self) {
        self.repaint_service().force_repaint();
    }
}

static RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

pub fn runtime() -> Runtime {
    if let Some(runtime) = RUNTIME.lock().unwrap().as_ref() {
        runtime.clone()
    } else {
        panic!("runtime not initialized")
    }
}

pub fn try_runtime() -> Option<Runtime> {
    RUNTIME.lock().unwrap().clone()
}

fn register_global(runtime: Option<Runtime>) {
    match runtime {
        Some(runtime) => {
            let mut global = RUNTIME.lock().unwrap();
            if global.is_some() {
                panic!("runtime already initialized");
            }
            global.replace(runtime);
        }
        None => {
            RUNTIME.lock().unwrap().take();
        }
    };
}

/// Spawn an async task that will result in
/// egui redraw upon its completion.
pub fn spawn<F>(task: F)
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    runtime().spawn_task(task);
}

/// Spawn an async task that will result in
/// egui redraw upon its completion. Upon
/// the task completion, the supplied [`Payload`]
/// will be populated with the task result.
pub fn spawn_with_result<R, F>(payload: &Payload<std::result::Result<R, Error>>, task: F)
where
    R: Clone + Send + 'static,
    F: Future<Output = Result<R>> + Send + 'static,
{
    runtime().spawn_task_with_result(payload, task);
}

/// Gracefully halt the runtime runtime. This is used
/// to shutdown tondid when the tondi-ng process exit
/// is an inevitable eventuality.
#[cfg(not(target_arch = "wasm32"))]
pub fn halt() {
    if let Some(runtime) = try_runtime() {
        runtime.try_send(Events::Exit).ok();
        runtime.tondi_service().clone().terminate();

        // 使用超时机制，防止halt操作卡住
        // 注意：这里不能使用 tokio::spawn，因为可能不在 Tokio runtime 上下文中
        let runtime_clone = runtime.clone();
        let handle = std::thread::spawn(move || {
            // 在新线程中创建临时的 Tokio runtime 来执行异步操作
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            
            rt.block_on(async {
                let timeout = tokio::time::Duration::from_secs(15);
                match tokio::time::timeout(timeout, runtime_clone.shutdown()).await {
                    Ok(_) => println!("Runtime shutdown completed"),
                    Err(_) => println!("Warning: Runtime shutdown timed out"),
                }
            });
        });

        // 添加超时等待，防止无限等待
        let start = std::time::Instant::now();
        let max_wait = std::time::Duration::from_secs(20);
        
        while !handle.is_finished() && start.elapsed() < max_wait {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        if !handle.is_finished() {
            println!("Warning: Runtime shutdown is taking too long, continuing with exit");
        }
    }
}

/// Attempt to halt the runtime runtime but exit the process
/// if it takes too long. This is used in attempt to shutdown
/// tondid if the tondi-ng process panics, which can result
/// in a still functioning zombie child process on unix systems.
#[cfg(not(target_arch = "wasm32"))]
pub fn abort() {
    const TIMEOUT: u128 = 5000;
    let flag = Arc::new(AtomicBool::new(false));
    let flag_ = flag.clone();
    let thread = std::thread::Builder::new()
        .name("halt".to_string())
        .spawn(move || {
            let start = std::time::Instant::now();
            while !flag_.load(Ordering::SeqCst) {
                if start.elapsed().as_millis() > TIMEOUT {
                    println!("halting...");
                    std::process::exit(1);
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        })
        .ok();

    halt();

    flag.store(true, Ordering::SeqCst);
    if let Some(thread) = thread {
        thread.join().unwrap();
    }

    #[cfg(feature = "console")]
    {
        println!("Press Enter to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    std::process::exit(1);
}
