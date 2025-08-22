use crate::imports::*;
use crate::runtime::Service;
pub use futures::{future::FutureExt, select, Future};
use tondi_wallet_core::api::*;
use tondi_wallet_core::events::Events as CoreWalletEvents;
#[allow(unused_imports)]
use tondi_wallet_core::rpc::{
    ConnectOptions, ConnectStrategy, NotificationMode, Rpc, RpcCtl, WrpcEncoding,
};
use tondi_wrpc_client::Resolver;
use workflow_core::runtime;
use std::time::Duration;

const ENABLE_PREEMPTIVE_DISCONNECT: bool = true;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[cfg(not(target_arch = "wasm32"))]
        use tondi_rpc_service::service::RpcCoreService;

        const LOG_BUFFER_LINES: usize = 4096;
        const LOG_BUFFER_MARGIN: usize = 128;
    }
}

// Add gRPC client module
pub mod grpc_client;
pub use grpc_client::TondiGrpcClient;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use std::path::PathBuf;

        pub mod config;
        pub use config::Config;
        pub mod daemon;
        pub mod inproc;
        pub mod logs;
        use logs::Log;
        pub use tondid_lib::args::Args;

        #[async_trait]
        pub trait Tondid {
            async fn start(self : Arc<Self>, config : Config) -> Result<()>;
            async fn stop(self : Arc<Self>) -> Result<()>;
        }

        #[derive(Debug, Clone)]
        pub enum TondidServiceEvents {
            StartInternalInProc { config: Config, network : Network },
            StartInternalAsDaemon { config: Config, network : Network },
            StartInternalAsPassiveSync { config: Config, network : Network },
            StartExternalAsDaemon { path: PathBuf, config: Config, network : Network },
            StartRemoteConnection { rpc_config : RpcConfig, network : Network },
            Stdout { line : String },
            Disable { network : Network },
            Exit,
        }

        pub fn update_logs_flag() -> &'static Arc<AtomicBool> {
            static FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();
            FLAG.get_or_init(||Arc::new(AtomicBool::new(false)))
        }

        pub fn update_metrics_flag() -> &'static Arc<AtomicBool> {
            static FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();
            FLAG.get_or_init(||Arc::new(AtomicBool::new(false)))
        }

    } else {

        #[derive(Debug)]
        pub enum TondidServiceEvents {
            StartRemoteConnection { rpc_config : RpcConfig, network : Network },
            Disable { network : Network },
            Exit,
        }

    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Context {}

pub struct TondiService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<TondidServiceEvents>,
    pub task_ctl: Channel<()>,
    pub network: Mutex<Network>,
    pub wallet: Arc<dyn WalletApi>,
    pub services_start_instant: Mutex<Option<Instant>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub tondid: Mutex<Option<Arc<dyn Tondid + Send + Sync + 'static>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub logs: Mutex<Vec<Log>>,
    pub connect_on_startup: Option<NodeSettings>,
}

impl TondiService {
    pub fn new(
        application_events: ApplicationEventsChannel,
        settings: &Settings,
        wallet: Option<Arc<dyn WalletApi>>,
    ) -> Self {
        // --
        // create wallet instance
        let storage = CoreWallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let wallet = wallet.unwrap_or_else(|| {
            Arc::new(
                CoreWallet::try_with_rpc(None, storage, Some(settings.node.network.into()))
                    .unwrap_or_else(|e| {
                        panic!("Failed to create wallet instance: {}", e);
                    }),
            )
        });

        // create service event channel
        let service_events = Channel::unbounded();

        if !settings.initialized {
            log_warn!("TondiService::new(): Settings are not initialized");
        }

        Self {
            // 对于devnet，总是尝试启动连接
            connect_on_startup: if settings.node.network == Network::Devnet {
                // 对于devnet，使用配置中的设置
                Some(settings.node.clone())
            } else {
                // 对于其他网络，使用原有逻辑
                settings.initialized.then(|| settings.node.clone())
            },
            application_events,
            service_events,
            task_ctl: Channel::oneshot(),
            network: Mutex::new(settings.node.network),
            wallet,
            services_start_instant: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            tondid: Mutex::new(None),
            #[cfg(not(target_arch = "wasm32"))]
            logs: Mutex::new(Vec::new()),
        }
    }

    pub async fn apply_node_settings(&self, node_settings: &NodeSettings) -> Result<()> {
        match TondidServiceEvents::from_node_settings(node_settings, None) {
            Ok(event) => {
                // log_trace!("TondiService::new(): emitting startup event: {:?}", event);
                self.service_events
                    .sender
                    .try_send(event)
                    .unwrap_or_else(|err| {
                        log_error!("TondidService error: {}", err);
                    });
            }
            Err(err) => {
                log_error!("TondidServiceEvents::try_from() error: {}", err);
            }
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn retain(&self, tondid: Arc<dyn Tondid + Send + Sync + 'static>) {
        self.tondid.lock().unwrap().replace(tondid);
    }

    pub async fn create_rpc_client(config: &RpcConfig, network: Network) -> Result<Rpc> {
        println!("[TONDI SERVICE] create_rpc_client 被调用");
        println!("[TONDI SERVICE] config: {:?}", config);
        println!("[TONDI SERVICE] network: {:?}", network);
        
        // 打印调用栈信息（简化版本）
        println!("[TONDI SERVICE] 调用来源检查...");
        if let RpcConfig::Grpc { url: Some(net_config) } = config {
            println!("[TONDI SERVICE] NetworkInterfaceConfig详细信息:");
            println!("[TONDI SERVICE]   kind: {:?}", net_config.kind);
            println!("[TONDI SERVICE]   custom: {:?}", net_config.custom);
        }
        
        match config {
            RpcConfig::Wrpc {
                url,
                encoding,
                resolver_urls,
            } => {
                println!("[TONDI SERVICE] 使用wRPC配置");
                let resolver_or_none = match url {
                    Some(_) => None,
                    None => {
                        if resolver_urls.is_some() {
                            Some(Resolver::new(Some(resolver_urls.clone().unwrap()), false))
                        } else {
                            Some(Resolver::new(resolver_urls.clone(), false))
                        }
                    }
                };

                let url = url.clone().unwrap_or_else(|| "127.0.0.1".to_string());
                let url =
                    TondiRpcClient::parse_url(url, *encoding, NetworkId::from(network).into())?;

                let wrpc_client = Arc::new(TondiRpcClient::new_with_args(
                    *encoding,
                    if resolver_or_none.is_some() {
                        None
                    } else {
                        Some(url.as_str())
                    },
                    resolver_or_none,
                    Some(NetworkId::from(network)),
                    None,
                )?);
                let rpc_ctl = wrpc_client.ctl().clone();
                let rpc_api: Arc<DynRpcApi> = wrpc_client;
                Ok(Rpc::new(rpc_api, rpc_ctl))
            }
            RpcConfig::Grpc { url } => {
                println!("[TONDI SERVICE] 使用gRPC配置");
                cfg_if! {
                    if #[cfg(not(target_arch = "wasm32"))] {
                        // Desktop version: supports gRPC
                        if let Some(network_interface) = url {
                            println!("[TONDI SERVICE] 使用指定的网络接口: {:?}", network_interface);
                            let grpc_client = TondiGrpcClient::connect(network_interface.clone(), network).await?;
                            let rpc_api: Arc<DynRpcApi> = Arc::new(grpc_client);
                            let rpc_ctl = RpcCtl::new();
                            // Set gRPC URL descriptor
                            let address: ContextualNetAddress = network_interface.clone().into();
                            rpc_ctl.set_descriptor(Some(format!("grpc://{}", address)));
                            println!("[TONDI SERVICE] gRPC客户端创建成功");
                            Ok(Rpc::new(rpc_api, rpc_ctl))
                        } else {
                            println!("[TONDI SERVICE] 使用默认网络接口");
                            // If no URL configured, use default configuration
                            let default_interface = NetworkInterfaceConfig::default();
                            println!("[TONDI SERVICE] 默认接口: {:?}", default_interface);
                            let grpc_client = TondiGrpcClient::connect(default_interface.clone(), network).await?;
                            let rpc_api: Arc<DynRpcApi> = Arc::new(grpc_client);
                            let rpc_ctl = RpcCtl::new();
                            // Set default gRPC URL descriptor
                            let address: ContextualNetAddress = default_interface.into();
                            rpc_ctl.set_descriptor(Some(format!("grpc://{}", address)));
                            println!("[TONDI SERVICE] 默认gRPC客户端创建成功");
                            Ok(Rpc::new(rpc_api, rpc_ctl))
                        }
                    } else {
                        // Web version: gRPC not supported, prompt to use wRPC
                        println!("[TONDI SERVICE] Web版本不支持gRPC");
                        Err(Error::custom("gRPC is not supported in Web/WASM version. Please use wRPC instead."))
                    }
                }
            }
        }
    }

    pub async fn connect_rpc_client(&self) -> Result<()> {
        if let Some(wallet) = self.core_wallet() {
            if let Ok(wrpc_client) = wallet.rpc_api().clone().downcast_arc::<TondiRpcClient>() {
                let options = ConnectOptions {
                    block_async_connect: false,
                    strategy: ConnectStrategy::Retry,
                    url: None,
                    connect_timeout: None,
                    retry_interval: Some(Duration::from_millis(3000)),
                };
                wrpc_client.connect(Some(options)).await?;
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if wallet
                        .rpc_api()
                        .clone()
                        .downcast_arc::<RpcCoreService>()
                        .is_ok()
                    {
                        wallet.rpc_ctl().signal_open().await?;
                    } else if let Ok(_grpc_client) = wallet.rpc_api().clone().downcast_arc::<TondiGrpcClient>() {
                        // gRPC客户端已经在创建时连接，这里不需要额外的连接步骤
                        println!("[TONDI SERVICE] gRPC客户端已连接，无需额外连接步骤");
                    } else {
                        unimplemented!("connect_rpc_client(): RPC client is not supported")
                    }
                }
            }
        }
        Ok(())
    }

    pub fn wallet(&self) -> Arc<dyn WalletApi> {
        self.wallet.clone()
    }

    pub fn core_wallet(&self) -> Option<Arc<CoreWallet>> {
        self.wallet.clone().downcast_arc::<CoreWallet>().ok()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn logs(&self) -> MutexGuard<'_, Vec<Log>> {
        self.logs.lock().unwrap()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn update_logs(&self, line: String) {
        {
            let mut logs = self.logs.lock().unwrap();
            if logs.len() > LOG_BUFFER_LINES {
                logs.drain(0..LOG_BUFFER_MARGIN);
            }
            logs.push(line.as_str().into());
        }

        if update_logs_flag().load(Ordering::SeqCst) {
            self.application_events
                .sender
                .send(crate::events::Events::UpdateLogs)
                .await
                .unwrap();
        }
    }

    pub fn rpc_url(&self) -> Option<String> {
        println!("[RPC URL DEBUG] rpc_url 被调用");
        if let Some(wallet) = self.core_wallet() {
            println!("[RPC URL DEBUG] wallet 存在，has_rpc: {}", wallet.has_rpc());
            if !wallet.has_rpc() {
                println!("[RPC URL DEBUG] wallet 没有 RPC，返回 None");
                None
            } else if let Ok(grpc_client) =
                wallet.rpc_api().clone().downcast_arc::<TondiGrpcClient>()
            {
                // gRPC客户端 - 优先显示gRPC连接
                println!("[RPC URL DEBUG] 检测到 gRPC 客户端");
                let raw_url = grpc_client.url();
                println!("[RPC URL DEBUG] gRPC 原始 URL: {:?}", raw_url);
                let formatted_url = raw_url.map(|url| {
                    // 检查URL是否已经包含scheme
                    if url.starts_with("grpc://") || url.starts_with("http://") || url.starts_with("https://") {
                        url
                    } else {
                        format!("grpc://{}", url)
                    }
                });
                println!("[RPC URL DEBUG] gRPC 格式化 URL: {:?}", formatted_url);
                formatted_url
            } else if let Ok(wrpc_client) =
                wallet.rpc_api().clone().downcast_arc::<TondiRpcClient>()
            {
                // wRPC客户端 - 作为fallback
                println!("[RPC URL DEBUG] 检测到 wRPC 客户端");
                let url = wrpc_client.url();
                println!("[RPC URL DEBUG] wRPC URL: {:?}", url);
                url
            } else {
                // 其他类型的RPC客户端
                println!("[RPC URL DEBUG] 未知的 RPC 客户端类型");
                None
            }
        } else {
            println!("[RPC URL DEBUG] wallet 不存在，返回 None");
            None
        }
    }

    fn is_wrpc_client(&self) -> bool {
        if let Some(wallet) = self.core_wallet() {
            wallet.has_rpc()
                && wallet
                    .rpc_api()
                    .clone()
                    .downcast_arc::<TondiRpcClient>()
                    .is_ok()
        } else {
            false
        }
    }

    async fn disconnect_rpc(&self) -> Result<()> {
        if let Some(wallet) = self.core_wallet() {
            if let Ok(wrpc_client) = wallet.rpc_api().clone().downcast_arc::<TondiRpcClient>() {
                wrpc_client.disconnect().await?;
            } else {
                wallet.rpc_ctl().signal_close().await?;
            }
        }
        Ok(())
    }

    pub async fn stop_all_services(&self) -> Result<()> {
        self.services_start_instant.lock().unwrap().take();

        if let Some(wallet) = self.core_wallet() {
            if !wallet.has_rpc() {
                return Ok(());
            }

            let preemptive_disconnect = ENABLE_PREEMPTIVE_DISCONNECT && self.is_wrpc_client();

            if preemptive_disconnect {
                self.disconnect_rpc().await?;
            }

            for service in crate::runtime::runtime().services().into_iter() {
                let instant = Instant::now();
                let service_name = service.name().to_string();
                
                // Add timeout mechanism to prevent individual services from hanging
                let detach_result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(5),
                    service.clone().detach_rpc()
                ).await;
                
                match detach_result {
                    Ok(Ok(_)) => {
                        if instant.elapsed().as_millis() > 1_000 {
                            log_warn!(
                                "WARNING: detach_rpc() for '{}' took {} msec",
                                service_name,
                                instant.elapsed().as_millis()
                            );
                        }
                    }
                    Ok(Err(e)) => {
                        log_warn!("Warning: detach_rpc() for '{}' failed: {}", service_name, e);
                    }
                    Err(_) => {
                        log_warn!("Warning: detach_rpc() for '{}' timed out after 5 seconds", service_name);
                    }
                }
            }

            if !preemptive_disconnect {
                self.disconnect_rpc().await?;
            }

            wallet.stop().await.expect("Unable to stop wallet");
            wallet.bind_rpc(None).await?;

            #[cfg(not(target_arch = "wasm32"))]
            {
                let tondid = self.tondid.lock().unwrap().take();
                if let Some(tondid) = tondid {
                    if let Err(err) = tondid.stop().await {
                        println!("error shutting down tondid: {}", err);
                    }
                }
            }
        } else {
            self.wallet().disconnect().await?;
        }
        Ok(())
    }

    pub async fn start_all_services(
        self: &Arc<Self>,
        rpc: Option<Rpc>,
        network: Network,
    ) -> Result<()> {
        self.services_start_instant
            .lock()
            .unwrap()
            .replace(Instant::now());

        *self.network.lock().unwrap() = network;

        if let (Some(rpc), Some(wallet)) = (rpc, self.core_wallet()) {
            let rpc_api = rpc.rpc_api().clone();

            wallet
                .set_network_id(&network.into())
                .expect("Can not change network id while the wallet is connected");

            wallet.bind_rpc(Some(rpc)).await.unwrap();
            wallet
                .start()
                .await
                .expect("Unable to start wallet service");

            for service in crate::runtime::runtime().services().into_iter() {
                service.attach_rpc(&rpc_api).await?;
            }

            Ok(())
        } else {
            // 如果没有提供RPC，尝试使用gRPC配置创建客户端
            cfg_if! {
                if #[cfg(not(target_arch = "wasm32"))] {
                    // 桌面版本：尝试使用gRPC配置
                    let default_net_config = NetworkInterfaceConfig::default();
                    println!("[CALL SITE 481 DEBUG] NetworkInterfaceConfig::default(): {:?}", default_net_config);
                    let grpc_config = RpcConfig::Grpc {
                        url: Some(default_net_config),
                    };
                                            println!("[CALL SITE 481] 调用 create_rpc_client");
                        match Self::create_rpc_client(&grpc_config, network).await {
                            Ok(grpc_rpc) => {
                                let rpc_api = grpc_rpc.rpc_api().clone();
                                
                                // 绑定gRPC客户端到钱包
                                if let Some(wallet) = self.core_wallet() {
                                    println!("[TONDI SERVICE DEBUG] 开始绑定gRPC客户端到钱包");
                                    wallet.bind_rpc(Some(grpc_rpc)).await.unwrap();
                                    println!("[TONDI SERVICE DEBUG] gRPC客户端已成功绑定到钱包");
                                    
                                    println!("[TONDI SERVICE DEBUG] 启动钱包服务");
                                    wallet
                                        .start()
                                        .await
                                        .expect("Unable to start wallet service");
                                    println!("[TONDI SERVICE DEBUG] 钱包服务已启动");
                                    
                                    // 手动触发连接事件，因为gRPC客户端不会自动触发
                                    println!("[TONDI SERVICE DEBUG] 手动触发 CoreWallet::Connect 事件");
                                    self.core_wallet_notify(CoreWalletEvents::Connect {
                                        network_id: network.into(),
                                        url: Some("grpc://8.210.45.192:16610".to_string()),
                                    }).unwrap();
                                    println!("[TONDI SERVICE DEBUG] CoreWallet::Connect 事件已发送");
                                } else {
                                    println!("[TONDI SERVICE DEBUG] 错误：core_wallet() 返回 None");
                                }

                                // 为所有服务附加gRPC API
                                for service in crate::runtime::runtime().services().into_iter() {
                                    service.attach_rpc(&rpc_api).await?;
                                }
                                
                                Ok(())
                            }
                        Err(_) => {
                            // gRPC失败，回退到默认连接
                            self.wallet()
                                .connect_call(ConnectRequest {
                                    url: None,
                                    network_id: network.into(),
                                    retry_on_error: true,
                                    block_async_connect: false,
                                    require_sync: false,
                                })
                                .await?;
                            Ok(())
                        }
                    }
                } else {
                    // Web版本：使用默认连接
                    self.wallet()
                        .connect_call(ConnectRequest {
                            url: None,
                            network_id: network.into(),
                            retry_on_error: true,
                            block_async_connect: false,
                            require_sync: false,
                        })
                        .await?;
                    Ok(())
                }
            }
        }
    }

    pub fn update_services(&self, node_settings: &NodeSettings, options: Option<RpcOptions>) {
        match TondidServiceEvents::from_node_settings(node_settings, options) {
            Ok(event) => {
                self.service_events
                    .sender
                    .try_send(event)
                    .unwrap_or_else(|err| {
                        println!("TondidService error: {}", err);
                    });
            }
            Err(err) => {
                println!("TondidServiceEvents::try_from() error: {}", err);
            }
        }
    }

    fn network(&self) -> Network {
        *self.network.lock().unwrap()
    }

    async fn handle_network_change(&self, network: Network) -> Result<()> {
        if network != self.network() {
            self.application_events
                .send(Events::NetworkChange(network))
                .await?;
        }

        Ok(())
    }

    pub async fn connect_all_services(&self) -> Result<()> {
        for service in crate::runtime::runtime().services().into_iter() {
            service.connect_rpc().await?;
        }

        Ok(())
    }

    pub async fn disconnect_all_services(&self) -> Result<()> {
        for service in crate::runtime::runtime().services().into_iter() {
            service.disconnect_rpc().await?;
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_storage(&self) {
        const STORAGE_UPDATE_DELAY: Duration = Duration::from_millis(3000);

        let options = StorageUpdateOptions::default()
            .if_not_present()
            .with_delay(STORAGE_UPDATE_DELAY);

        runtime().update_storage(options);
    }

    async fn handle_event(self: &Arc<Self>, event: TondidServiceEvents) -> Result<bool> {
        match event {
            #[cfg(not(target_arch = "wasm32"))]
            TondidServiceEvents::Stdout { line } => {
                let wallet = self.core_wallet().ok_or(Error::WalletIsNotLocal)?;
                if !wallet.utxo_processor().is_synced() {
                    wallet
                        .utxo_processor()
                        .sync_proc()
                        .handle_stdout(&line)
                        .await?;
                }

                self.update_logs(line).await;
            }

            #[cfg(not(target_arch = "wasm32"))]
            TondidServiceEvents::StartInternalInProc { config, network } => {
                self.stop_all_services().await?;

                self.handle_network_change(network).await?;

                println!("[TONDI] 启动进程内节点...");
                let tondid = Arc::new(inproc::InProc::default());
                self.retain(tondid.clone());

                // 克隆config以避免所有权问题
                let config_clone = config.clone();
                tondid.clone().start(config).await.unwrap();
                println!("[TONDI] 进程内节点启动成功");

                // 等待更长时间让节点完全启动和初始化
                println!("[TONDI] 等待节点完全启动和初始化...");
                tokio::time::sleep(Duration::from_secs(10)).await;
                println!("[TONDI] 节点初始化完成，尝试连接gRPC...");

                // 尝试使用gRPC连接到本地节点
                let grpc_config = RpcConfig::Grpc {
                    url: Some(config_clone.grpc_network_interface.clone()),
                };
                
                match Self::create_rpc_client(&grpc_config, network).await {
                    Ok(grpc_rpc) => {
                        println!("[TONDI] 成功连接到本地gRPC端点");
                        let rpc_api = grpc_rpc.rpc_api().clone();
                        let rpc_ctl = RpcCtl::new();
                        let rpc = Rpc::new(rpc_api, rpc_ctl.clone());

                        self.start_all_services(Some(rpc), network).await?;
                        self.connect_rpc_client().await?;
                        
                        // 等待一下让服务完全启动
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        println!("[TONDI] 所有服务启动完成");
                    }
                    Err(e) => {
                        println!("[TONDI] gRPC连接失败: {}, 回退到进程内RPC", e);
                        // 回退到进程内RPC
                        let rpc_api = tondid
                            .rpc_core_services()
                            .expect("Unable to obtain inproc rpc api");
                        let rpc_ctl = RpcCtl::new();
                        let rpc = Rpc::new(rpc_api, rpc_ctl.clone());

                        self.start_all_services(Some(rpc), network).await?;
                        self.connect_rpc_client().await?;
                        
                        // 等待一下让服务完全启动
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        println!("[TONDI] 所有服务启动完成（进程内RPC模式）");
                    }
                }

                self.update_storage();
            }
            #[cfg(not(target_arch = "wasm32"))]
            TondidServiceEvents::StartInternalAsDaemon { config, network } => {
                self.stop_all_services().await?;

                self.handle_network_change(network).await?;

                let tondid = Arc::new(daemon::Daemon::new(None, &self.service_events));
                self.retain(tondid.clone());
                tondid.clone().start(config).await.unwrap();

                // 根据网络类型动态选择RPC配置
                let rpc_config = if network == Network::Devnet {
                    // 对于devnet，优先使用gRPC配置
                    println!("[TONDI SERVICE] Devnet模式，使用gRPC配置");
                    RpcConfig::Grpc {
                        url: Some(NetworkInterfaceConfig {
                            kind: NetworkInterfaceKind::Custom,
                            custom: "8.210.45.192:16610".parse().unwrap(),
                        }),
                    }
                } else {
                    // 对于其他网络，使用wRPC配置
                    println!("[TONDI SERVICE] 使用wRPC配置");
                    RpcConfig::Wrpc {
                        url: Some("127.0.0.1".to_string()),
                        encoding: WrpcEncoding::Borsh,
                        resolver_urls: None,
                    }
                };

                let rpc = Self::create_rpc_client(&rpc_config, network).await
                    .expect("Tondid Service - unable to create wRPC client");
                self.start_all_services(Some(rpc), network).await?;
                self.connect_rpc_client().await?;

                self.update_storage();
            }
            #[cfg(not(target_arch = "wasm32"))]
            TondidServiceEvents::StartInternalAsPassiveSync { config, network } => {
                self.stop_all_services().await?;

                self.handle_network_change(network).await?;

                let tondid = Arc::new(daemon::Daemon::new(None, &self.service_events));
                self.retain(tondid.clone());
                tondid.clone().start(config).await.unwrap();

                // 根据网络类型动态选择RPC配置
                let rpc_config = if network == Network::Devnet {
                    // 对于devnet，优先使用gRPC配置
                    println!("[TONDI SERVICE] Devnet模式，使用gRPC配置");
                    RpcConfig::Grpc {
                        url: Some(NetworkInterfaceConfig {
                            kind: NetworkInterfaceKind::Custom,
                            custom: "8.210.45.192:16610".parse().unwrap(),
                        }),
                    }
                } else {
                    // 对于其他网络，使用wRPC配置
                    println!("[TONDI SERVICE] 使用wRPC配置");
                    RpcConfig::Wrpc {
                        url: None,
                        encoding: WrpcEncoding::Borsh,
                        resolver_urls: None,
                    }
                };

                let rpc = Self::create_rpc_client(&rpc_config, network).await
                    .expect("Tondid Service - unable to create RPC client");
                self.start_all_services(Some(rpc), network).await?;
                self.connect_rpc_client().await?;

                self.update_storage();
            }
            #[cfg(not(target_arch = "wasm32"))]
            TondidServiceEvents::StartExternalAsDaemon {
                path,
                config,
                network,
            } => {
                self.stop_all_services().await?;

                self.handle_network_change(network).await?;

                let tondid = Arc::new(daemon::Daemon::new(Some(path), &self.service_events));
                self.retain(tondid.clone());

                tondid.clone().start(config).await.unwrap();

                // 根据网络类型动态选择RPC配置
                let rpc_config = if network == Network::Devnet {
                    // 对于devnet，优先使用gRPC配置
                    println!("[TONDI SERVICE] Devnet模式，使用gRPC配置");
                    RpcConfig::Grpc {
                        url: Some(NetworkInterfaceConfig {
                            kind: NetworkInterfaceKind::Custom,
                            custom: "8.210.45.192:16610".parse().unwrap(),
                        }),
                    }
                } else {
                    // 对于其他网络，使用wRPC配置
                    println!("[TONDI SERVICE] 使用wRPC配置");
                    RpcConfig::Wrpc {
                        url: None,
                        encoding: WrpcEncoding::Borsh,
                        resolver_urls: None,
                    }
                };

                let rpc = Self::create_rpc_client(&rpc_config, network).await
                    .expect("Tondid Service - unable to create wRPC client");
                self.start_all_services(Some(rpc), network).await?;
                self.connect_rpc_client().await?;

                self.update_storage();
            }
            TondidServiceEvents::StartRemoteConnection {
                rpc_config,
                network,
            } => {
                if runtime::is_chrome_extension() {
                    self.stop_all_services().await?;

                    self.handle_network_change(network).await?;
                    self.wallet().change_network_id(network.into()).await.ok();

                    self.start_all_services(None, network).await?;
                    self.connect_rpc_client().await?;
                } else {
                    self.stop_all_services().await?;

                    self.handle_network_change(network).await?;

                    let rpc = Self::create_rpc_client(&rpc_config, network).await
                        .expect("Tondid Service - unable to create wRPC client");
                    self.start_all_services(Some(rpc), network).await?;
                    self.connect_rpc_client().await?;
                }
            }

            TondidServiceEvents::Disable { network } => {
                if let Some(wallet) = self.core_wallet() {
                    self.stop_all_services().await?;

                    self.handle_network_change(network).await?;

                    if wallet.is_open() {
                        wallet.close().await.ok();
                    }

                    // re-apply network id to allow wallet
                    // to be opened offline in disconnected
                    // mode by changing network id in settings
                    wallet.set_network_id(&network.into()).ok();
                } else if runtime::is_chrome_extension() {
                    self.stop_all_services().await?;
                    self.wallet().wallet_close().await.ok();
                    self.handle_network_change(network).await?;
                    self.wallet().change_network_id(network.into()).await.ok();
                }
            }

            TondidServiceEvents::Exit => {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn handle_multiplexer(
        &self,
        event: Box<tondi_wallet_core::events::Events>,
    ) -> Result<()> {
        // use tondi_wallet_core::events::Events as CoreWalletEvents;

        match *event {
            CoreWalletEvents::DaaScoreChange { .. } => {}
            CoreWalletEvents::Connect { .. } => {
                self.connect_all_services().await?;

                // self.wallet().
            }
            CoreWalletEvents::Disconnect { .. } => {
                self.disconnect_all_services().await?;
            }
            _ => {
                // println!("wallet event: {:?}", event);
            }
        }
        self.application_events
            .sender
            .send(crate::events::Events::Wallet { event })
            .await
            .unwrap();
        // }

        Ok(())
    }

    fn core_wallet_notify(&self, event: tondi_wallet_core::events::Events) -> Result<()> {
        self.application_events
            .sender
            .try_send(crate::events::Events::Wallet {
                event: Box::new(event),
            })?;
        // .try_send(Box::new(event))?;
        Ok(())
    }

    fn notify(&self, event: crate::events::Events) -> Result<()> {
        self.application_events.sender.try_send(event)?;
        Ok(())
    }
}

#[async_trait]
impl Service for TondiService {
    fn name(&self) -> &'static str {
        "tondi-service"
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let _application_events_sender = self.application_events.sender.clone();

        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT
        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT
        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT
        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT
        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT
        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT
        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT
        // ^ TODO: - CHECK IF THE WALLET IS OPEN, GET WALLET CONTEXT

        let status = if runtime::is_chrome_extension() {
            self.wallet().get_status(Some("tondi-ng")).await.ok()
        } else {
            None
        };

        if let Some(status) = status {
            let GetStatusResponse {
                is_connected,
                is_open: _,
                is_synced,
                url,
                is_wrpc_client: _,
                network_id,
                context,
                selected_account_id,
                wallet_descriptor,
                account_descriptors,
            } = status;
            
            println!("[TONDI SERVICE DEBUG] GetStatusResponse: is_connected={}, url={:?}, network_id={:?}", is_connected, url, network_id);

            if let Some(context) = context {
                let _context = Context::try_from_slice(&context)?;

                if is_connected {
                    let network_id = network_id.unwrap_or_else(|| self.network().into());
                    
                    println!("[TONDI SERVICE DEBUG] GetStatusResponse.is_connected=true, 发送 CoreWallet::Connect 事件");
                    println!("[TONDI SERVICE DEBUG] Connect 事件参数: network_id={:?}, url={:?}", network_id, url);

                    self.core_wallet_notify(CoreWalletEvents::Connect {
                        network_id,
                        url: url.clone(),
                    })
                    .unwrap();

                    // ^ TODO - Get appropriate `server_version`
                    let server_version = Default::default();
                    // let event = Box::new(CoreWalletEvents::ServerStatus {
                    //     is_synced,
                    //     network_id,
                    //     url,
                    //     server_version,
                    // });
                    // self.application_events
                    //     .sender
                    //     .try_send(crate::events::Events::Wallet { event })
                    //     // .await
                    //     .unwrap();

                    self.core_wallet_notify(CoreWalletEvents::ServerStatus {
                        is_synced,
                        network_id,
                        url,
                        server_version,
                    })
                    .unwrap();
                }

                if let (Some(wallet_descriptor), Some(account_descriptors)) =
                    (wallet_descriptor, account_descriptors)
                {
                    self.core_wallet_notify(CoreWalletEvents::WalletOpen {
                        wallet_descriptor: Some(wallet_descriptor),
                        account_descriptors: Some(account_descriptors),
                    })
                    .unwrap();
                }

                if let Some(selected_account_id) = selected_account_id {
                    self.core_wallet_notify(CoreWalletEvents::AccountSelection {
                        id: Some(selected_account_id),
                    })
                    .unwrap();

                    self.notify(crate::events::Events::ChangeSection(TypeId::of::<
                        crate::modules::account_manager::AccountManager,
                    >()))
                    .unwrap();
                }

                // ^ MOVE THIS FUNCTION TO "bootstrap()"
                // ^ MOVE THIS FUNCTION TO "bootstrap()"
                // ^ MOVE THIS FUNCTION TO "bootstrap()"
            } else {
                // new instance - emit startup event
                if let Some(node_settings) = self.connect_on_startup.as_ref() {
                    // 即使有connect_on_startup，也优先尝试启动本地集成节点
                    cfg_if! {
                        if #[cfg(not(target_arch = "wasm32"))] {
                            if node_settings.node_kind == TondidNodeKind::IntegratedInProc {
                                println!("[TONDI] 检测到集成节点配置，启动本地集成节点...");
                                let config = Config::from(node_settings.clone());
                                let event = TondidServiceEvents::StartInternalInProc { 
                                    config: config.clone(), 
                                    network: node_settings.network 
                                };
                                self.service_events.sender.try_send(event).unwrap_or_else(|err| {
                                    println!("[TONDI] 无法发送启动事件: {}", err);
                                });
                            } else {
                                // 非集成节点，使用原有逻辑
                                self.apply_node_settings(node_settings).await?;
                            }
                        } else {
                            // Web版本，使用原有逻辑
                            self.apply_node_settings(node_settings).await?;
                        }
                    }
                } else {
                    // 如果没有配置启动连接，根据网络类型决定启动策略
                    cfg_if! {
                        if #[cfg(not(target_arch = "wasm32"))] {
                            match Network::default() {
                                Network::Devnet => {
                                    println!("[TONDI] Devnet模式，尝试连接远程节点...");
                                    // 对于devnet，使用远程连接配置
                                    let event = TondidServiceEvents::StartRemoteConnection { 
                                        rpc_config: RpcConfig::Grpc {
                                            url: Some(NetworkInterfaceConfig {
                                                kind: NetworkInterfaceKind::Custom,
                                                custom: "8.210.45.192:16610".parse().unwrap(),
                                            }),
                                        },
                                        network: Network::Devnet 
                                    };
                                    self.service_events.sender.try_send(event).unwrap_or_else(|err| {
                                        println!("[TONDI] 无法发送启动事件: {}", err);
                                    });
                                }
                                _ => {
                                    println!("[TONDI] 自动启动本地集成节点...");
                                    let config = Config::from_network(Network::Mainnet);
                                    let event = TondidServiceEvents::StartInternalInProc { 
                                        config: config.clone(), 
                                        network: Network::Mainnet 
                                    };
                                    self.service_events.sender.try_send(event).unwrap_or_else(|err| {
                                        println!("[TONDI] 无法发送启动事件: {}", err);
                                    });
                                }
                            }
                        }
                    }
                }

                // new instance - setup new context
                let context = Context {};
                self.wallet()
                    .retain_context("tondi-ng", Some(borsh::to_vec(&context)?))
                    .await?;
            }
        } else {
            // new instance - emit startup event
            if let Some(node_settings) = self.connect_on_startup.as_ref() {
                self.apply_node_settings(node_settings).await?;
            }
        }
        // else if let Some(node_settings) = self.connect_on_startup.as_ref() {
        //     // self.update_services(node_settings, None);
        //     self.apply_node_settings(node_settings).await?;
        // }

        if let Some(wallet) = self.core_wallet() {
            // wallet.multiplexer().channel()
            let wallet_events = wallet.multiplexer().channel();

            loop {
                select! {
                    msg = wallet_events.recv().fuse() => {
                    // msg = wallet.multiplexer().channel().recv().fuse() => {
                        if let Ok(event) = msg {
                            self.handle_multiplexer(event).await?;
                        } else {
                            break;
                        }
                    }

                    msg = self.as_ref().service_events.receiver.recv().fuse() => {
                        if let Ok(event) = msg {
                            if self.handle_event(event).await? {
                                break;
                            }

                        } else {
                            break;
                        }
                    }
                }
            }
        } else {
            loop {
                select! {
                    // msg = wallet_events.recv().fuse() => {
                    // // msg = wallet.multiplexer().channel().recv().fuse() => {
                    //     if let Ok(event) = msg {
                    //         self.handle_multiplexer(event).await?;
                    //     } else {
                    //         break;
                    //     }
                    // }

                    msg = self.as_ref().service_events.receiver.recv().fuse() => {
                        if let Ok(event) = msg {
                            if self.handle_event(event).await? {
                                break;
                            }

                        } else {
                            break;
                        }
                    }
                }
            }
        };

        self.stop_all_services().await?;
        self.task_ctl.send(()).await.unwrap();

        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(TondidServiceEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}

impl TondidServiceEvents {
    pub fn from_node_settings(
        node_settings: &NodeSettings,
        options: Option<RpcOptions>,
    ) -> Result<Self> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // 桌面版本：所有模式都使用gRPC配置
                match &node_settings.node_kind {
                    TondidNodeKind::Disable => {
                        Ok(TondidServiceEvents::Disable { network : node_settings.network })
                    }
                    TondidNodeKind::IntegratedInProc => {
                        // 对于集成模式，也使用gRPC配置
                        Ok(TondidServiceEvents::StartInternalInProc { config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    TondidNodeKind::IntegratedAsDaemon => {
                        // 对于集成守护进程模式，也使用gRPC配置
                        Ok(TondidServiceEvents::StartInternalAsDaemon { config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    TondidNodeKind::IntegratedAsPassiveSync => {
                        // 对于被动同步模式，也使用gRPC配置
                        Ok(TondidServiceEvents::StartInternalAsPassiveSync { config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    TondidNodeKind::ExternalAsDaemon => {
                        let path = node_settings.tondid_daemon_binary.clone();
                        Ok(TondidServiceEvents::StartExternalAsDaemon { path : PathBuf::from(path), config : Config::from(node_settings.clone()), network : node_settings.network })
                    }
                    TondidNodeKind::Remote => {
                        // 远程模式使用gRPC配置
                        Ok(TondidServiceEvents::StartRemoteConnection { rpc_config : RpcConfig::from_node_settings(node_settings,options), network : node_settings.network })
                    }
                }

            } else {
                // Web版本：只支持远程模式，使用wRPC配置
                match &node_settings.node_kind {
                    TondidNodeKind::Disable => {
                        Ok(TondidServiceEvents::Disable { network : node_settings.network })
                    }
                    TondidNodeKind::Remote => {
                        // Web版本强制使用wRPC
                        let mut web_settings = node_settings.clone();
                        web_settings.rpc_kind = RpcKind::Wrpc;
                        Ok(TondidServiceEvents::StartRemoteConnection { rpc_config : RpcConfig::from_node_settings(&web_settings,options), network : node_settings.network })
                    }
                }
            }
        }
    }
}
