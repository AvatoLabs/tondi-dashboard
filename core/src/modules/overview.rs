use workflow_core::runtime::is_wasm;

#[cfg(not(feature = "lean"))]
use tondi_metrics_core::{Metric,MetricGroup};
#[cfg(not(feature = "lean"))]
use egui_plot::{
    Legend,
    Line,
    LineStyle,
    Plot,
    PlotPoints,
};

use crate::imports::*;

pub struct Overview {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Overview {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Overview {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Default
    }


    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        if core.device().single_pane() {
            self.render_details(core, ui);
        } else {
            
            cfg_if! {
                if #[cfg(not(feature = "lean"))] {

                    let width = ui.available_width();
                    
                    SidePanel::left("overview_left")
                        .exact_width(width*0.5)
                        .resizable(false)
                        .show_separator_line(true)
                        .show_inside(ui, |ui| {
                            egui::ScrollArea::vertical()
                            .id_salt("overview_metrics")
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                self.render_stats(core,ui);
                            });
                        });
                    
                    SidePanel::right("overview_right")
                        .exact_width(width*0.5)
                        .frame(Frame::default().fill(Color32::TRANSPARENT))
                        .resizable(false)
                        .show_separator_line(false)
                        .show_inside(ui, |ui| {
                            self.render_details(core, ui);
                        });
                }
        
            }
        }


    }
}

impl Overview {

    #[cfg(not(feature = "lean"))]
    fn render_stats(&mut self, core: &mut Core, ui : &mut Ui) {

        let node_info = if let Some(node_info) = &core.node_info {
            format!(" - {}", node_info)
        } else {
            "".to_string()
        };

        CollapsingHeader::new(format!("{}{}",i18n("Tondi p2p Node"), node_info))
            .default_open(true)
            .show(ui, |ui| {

                if core.state().is_connected() {
                    self.render_graphs(core,ui);
                } else {
                    ui.label(i18n("Not connected"));
                }
            });

        ui.add_space(48.);
    }

    fn render_details(&mut self, core: &mut Core, ui : &mut Ui) {

        egui::ScrollArea::vertical()
            .id_salt("overview_metrics")
            .auto_shrink([false; 2])
            .show(ui, |ui| {

                if core.settings.market_monitor {
                    if let Some(market) = core.market.as_ref() {

                        CollapsingHeader::new(i18n("Market"))
                            .default_open(true)
                            .show(ui, |ui| {

                                if let Some(price_list) = market.price.as_ref() {
                                    let mut symbols = price_list.keys().collect::<Vec<_>>();
                                    symbols.sort();
                                    symbols.into_iter().for_each(|symbol| {
                                        if let Some(data) = price_list.get(symbol) {
                                            let symbol = symbol.to_uppercase();
                                            CollapsingHeader::new(symbol.as_str())
                                                .default_open(true)
                                                .show(ui, |ui| {
                                                    Grid::new("market_price_info_grid")
                                                        .num_columns(2)
                                                        .spacing([16.0,4.0])
                                                        .show(ui, |ui| {
                                                            let MarketData { price, volume, change, market_cap , precision } = *data;
                                                            ui.label(i18n("Price"));
                                                            ui.colored_label(theme_color().market_default_color, RichText::new(format_currency_with_symbol(price, precision, symbol.as_str()))); // 
                                                            ui.end_row();

                                                            ui.label(i18n("24h Change"));
                                                            if change > 0. { 
                                                                ui.colored_label(theme_color().market_up_color, RichText::new(format!("+{:.2}%  ",change)));
                                                            } else { 
                                                                ui.colored_label(theme_color().market_down_color, RichText::new(format!("{:.2}%  ",change)));
                                                            };
                                                            ui.end_row();

                                                            ui.label(i18n("Volume"));
                                                            ui.colored_label(theme_color().market_default_color, RichText::new(format!("{} {}",volume.trunc().separated_string(),symbol.to_uppercase())));
                                                            ui.end_row();

                                                            ui.label(i18n("Market Cap"));
                                                            ui.colored_label(theme_color().market_default_color, RichText::new(format!("{} {}",market_cap.trunc().separated_string(),symbol.to_uppercase())));
                                                            ui.end_row();
                                                        });

                                                });
                                        }
                                    })
                                }
                            });
                        }
                    }

                #[cfg(not(target_arch = "wasm32"))]
                CollapsingHeader::new(i18n("Tondi Dashboard"))
                    .default_open(true)
                    .show(ui, |ui| {
                        use egui_phosphor::light::CLOUD;

                        ui.hyperlink_to_tab(
                            format!("• {CLOUD} {}",i18n("Tondi Dashboard online")),
                            "https://tondi-ng.org"
                        );
                    });

                match core.settings.node.network {
                    Network::Mainnet => {
                        CollapsingHeader::new(i18n("Mainnet"))
                            .default_open(true)
                            .show(ui, |ui| {
                                CollapsingHeader::new(i18n("Resources"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        use egui_phosphor::light::{CHART_SCATTER,DATABASE};
                
                                        ui.hyperlink_to_tab(
                                            format!("• {DATABASE} {}",i18n("Explorer")),
                                            "https://explorer.tondi.org/",
                                        );
                                        ui.hyperlink_to_tab(
                                            format!("• {CHART_SCATTER} {}",i18n("Statistics")),
                                            "https://kas.fyi",
                                        );
                                        // ui.hyperlink_to_tab(
                                        //     format!("• {DISCORD_LOGO} {}",i18n("Discord")),
                                        //     "https://discord.com/invite/kS3SK5F36R",
                                        // );
                
                                    });
                                self.render_network_info(core, ui);
                                self.render_fee_rate(core, ui);
                            });
                    }
                    Network::Testnet => {
                        CollapsingHeader::new(i18n("Testnet"))
                            .default_open(true)
                            .show(ui, |ui| {
                                CollapsingHeader::new(i18n("Resources"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        use egui_phosphor::light::{HAND_COINS,DATABASE};
                
                                        ui.hyperlink_to_tab(
                                            format!("• {DATABASE} {}",i18n("Explorer")),
                                            "https://explorer-tn10.tondi.org/",
                                        );
                                        ui.hyperlink_to_tab(
                                            format!("• {HAND_COINS} {}",i18n("Faucet")),
                                            "https://faucet-testnet.tondinet.io",
                                        );
                
                                    });
                                self.render_network_info(core, ui);
                                self.render_fee_rate(core, ui);
                            });
                    }
                    Network::Devnet => {
                        CollapsingHeader::new(i18n("Devnet"))
                            .default_open(true)
                            .show(ui, |ui| {
                                CollapsingHeader::new(i18n("Resources"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        use egui_phosphor::light::{DATABASE};
                
                                        ui.hyperlink_to_tab(
                                            format!("• {DATABASE} {}",i18n("Explorer")),
                                            "https://explorer-dev11.tondi.org/",
                                        );
                                    });
                                self.render_network_info(core, ui);
                                self.render_fee_rate(core, ui);
                            });
                    }
                }

                CollapsingHeader::new(i18n("Developer Resources"))
                    .default_open(true)
                    .show(ui, |ui| {
                        

                        ui.hyperlink_to_tab(
                            format!("• {}",i18n("Tondi Dashboard on GitHub")),
                            "https://github.com/aspectron/tondi-ng"
                        );
                        ui.hyperlink_to_tab(
                            format!("• {}",i18n("Tondi Client on GitHub")),
                            "https://github.com/AvatoLabs/Tondi",
                        );
                        ui.hyperlink_to_tab(
                            format!("• {}",i18n("Tondi Integration Guide")),
                            "https://tondi.aspectron.org",
                        );
                        ui.hyperlink_to_tab(
                            format!("• {}",i18n("NPM Modules for NodeJS")),
                            "https://www.npmjs.com/package/tondi",
                        );
                        ui.hyperlink_to_tab(
                            format!("• {}",i18n("WASM SDK for JavaScript and TypeScript")),
                            "https://aspectron.org/en/projects/tondi-wasm.html",
                        );
                        ui.hyperlink_to_tab(
                            format!("• {}",i18n("Rust Wallet SDK")),
                            "https://docs.rs/tondi-wallet-core/",
                        );
                        ui.hyperlink_to_tab(
                            format!("• {}",i18n("Discord")),
                            "https://discord.com/invite/kS3SK5F36R",
                        );
                    });

                if let Some(release) = core.release.as_ref() {
                    let is_greater = is_version_greater(crate::app::VERSION, release.version.as_str()).ok().unwrap_or(false);
                    if is_wasm() || !is_greater {
                        CollapsingHeader::new(i18n("Redistributables"))
                            .id_salt("redistributables")
                            .default_open(true)
                            .show(ui, |ui| {
                                release.assets.iter().for_each(|asset| {
                                    Hyperlink::from_label_and_url(
                                        format!("• {}", asset.name),
                                        asset.browser_download_url.clone(),
                                    ).open_in_new_tab(true).ui(ui);
                                });
                            });
                    } else {
                        CollapsingHeader::new(RichText::new(format!("{} {}",i18n("Update Available to version"), release.version)).color(theme_color().warning_color).strong())
                            .id_salt("redistributables-update")
                            .default_open(true)
                            .show(ui, |ui| {

                                if let Some(html_url) = &release.html_url {
                                    Hyperlink::from_label_and_url(
                                        format!("• {} {}", i18n("GitHub Release"), release.version),
                                        html_url,
                                    ).open_in_new_tab(true).ui(ui);
                                }

                                release.assets.iter().for_each(|asset| {
                                    Hyperlink::from_label_and_url(
                                        format!("• {}", asset.name),
                                        asset.browser_download_url.clone(),
                                    ).open_in_new_tab(true).ui(ui);
                                });

                            });

                    }
                }

                CollapsingHeader::new(i18n("Build"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(Label::new(format!("Tondi Dashboard v{}-{} + Tondi Client {}", env!("CARGO_PKG_VERSION"),crate::app::GIT_DESCRIBE, tondi_version())));
                        // if ui.add(Label::new(format!("Tondi Dashboard v{}-{} + Tondi Client v{}", env!("CARGO_PKG_VERSION"),crate::app::GIT_DESCRIBE, tondi_wallet_core::version())).sense(Sense::click())).clicked() {
                        //     core.select::<modules::Changelog>();
                        // }
                        // ui.label(format!("Timestamp: {}", crate::app::BUILD_TIMESTAMP));
                        ui.label(i18n_args("Timestamp: {timestamp}", &[("timestamp", crate::app::BUILD_TIMESTAMP)]));
                        ui.label(format!("rustc {}-{} {}  llvm {}",
                            crate::app::RUSTC_SEMVER,
                            crate::app::RUSTC_COMMIT_HASH.chars().take(8).collect::<String>(),
                            crate::app::RUSTC_CHANNEL,
                            crate::app::RUSTC_LLVM_VERSION,
                        ));
                        ui.label(i18n_args("Architecture {arch}", &[("arch", crate::app::CARGO_TARGET_TRIPLE)]));
                    });

                if let Some(system) = runtime().system() {
                    system.render(ui);
                }

                #[cfg(not(target_arch = "wasm32"))]
                core.storage.render(ui);
        
                CollapsingHeader::new(i18n("License Information"))
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.vertical(|ui|{
                            ui.label("Tondi Client");
                            ui.label("Copyright (c) 2025 Tondi Foundation");
                            ui.label("License: ISC");
                            ui.hyperlink_url_to_tab("https://github.com/AvatoLabs/Tondi");
                            ui.label("");
                            ui.label("Tondi Dashboard");
                            ui.label("Copyright (c) 2025 Tondi Foundation");
                            ui.label("License: MIT (RESTRICTED)");
                            ui.hyperlink_url_to_tab("https://github.com/Tondi-Foundation/tondi-dashboard");
                            ui.label("");
                            ui.label("WORKFLOW-RS");
                            ui.label("Copyright (c) 2025 Tondi Foundation");
                            ui.label("License: MIT or Apache 2.0");
                            ui.hyperlink_url_to_tab("https://github.com/workflow-rs/workflow-rs");
                            ui.label("");
                            ui.label("EGUI");
                            ui.label("Copyright (c) 2024 Rerun");
                            ui.label("License: MIT or Apache 2.0");
                            ui.hyperlink_url_to_tab("https://github.com/emilk/egui");
                            ui.label("");
                            ui.label("PHOSPHOR ICONS");
                            ui.label("Copyright (c) 2024 ");
                            ui.label("License: MIT");
                            ui.hyperlink_url_to_tab("https://phosphoricons.com/");
                            ui.label("");
                            ui.label("Illustration Art");
                            ui.label("Copyright (c) 2023 Rhubarb Media");
                            ui.label("License: CC BY 4.0");
                            ui.hyperlink_url_to_tab("https://rhubarbmedia.ca/");
                            ui.label("");
                        });
                    });
            });
    }

    fn render_network_info(&self, core: &Core, ui : &mut Ui) {

        CollapsingHeader::new(i18n("Statistics"))
            .default_open(true)
            .show(ui, |ui| {
             // ui.label(format!("Network Pressure: ~{}%", core.network_pressure.capacity()));
                ui.label(i18n_args("Network Pressure: ~{number}%", &[("number", core.network_pressure.capacity().to_string())]));
            });
    }

    fn render_fee_rate(&self, core: &Core, ui : &mut Ui) {

        if let Some(fees) = core.feerate.as_ref() {
            let (low,med,high) = if core.network_pressure.below_capacity() {
                (1.0,1.0,1.0)
            } else {
                (fees.low.value().feerate, fees.economic.value().feerate, fees.priority.value().feerate)
            };
            let low_kas = sompi_to_tondi_string_with_suffix((low * BASIC_TRANSACTION_MASS as f64) as u64, &core.settings.node.network.into());
            let med_kas = sompi_to_tondi_string_with_suffix((med * BASIC_TRANSACTION_MASS as f64) as u64, &core.settings.node.network.into());
            let high_kas = sompi_to_tondi_string_with_suffix((high * BASIC_TRANSACTION_MASS as f64) as u64, &core.settings.node.network.into());
            CollapsingHeader::new(i18n("Fee Market"))
                .default_open(true)
                .show(ui, |ui| {
                //  ui.label(format!("Low: {} SOMPI/g;  ~{}/tx", format_with_precision(low), low_kas));
                //  ui.label(format!("Economic: {} SOMPI/g;  ~{}/tx", format_with_precision(med),med_kas));
                //  ui.label(format!("Priority: {} SOMPI/g;  ~{}/tx", format_with_precision(high),high_kas));
                    ui.label(i18n_args("Low: {low} SOMPI/g;  ~{low_kas}/tx", &[("low", format_with_precision(low)), ("low_kas", low_kas)]));
                    ui.label(i18n_args("Economic: {med} SOMPI/g;  ~{med_kas}/tx", &[("med", format_with_precision(med)), ("med_kas", med_kas)]));
                    ui.label(i18n_args("Priority: {high} SOMPI/g;  ~{high_kas}/tx", &[("high", format_with_precision(high)), ("high_kas", high_kas)]));
                });
        }
    }

    #[cfg(not(feature = "lean"))]
    fn render_graphs(&mut self, core: &mut Core, ui : &mut Ui) {

        let mut metric_iter = METRICS.iter();

        if let Some(_snapshot) = core.metrics() {
            let view_width = ui.available_width();

            if view_width < 200. {
                return;
            }
            // 增加图表宽度，减少边距，让metrics部分更宽
            const GRAPH_WIDTH: f32 = 160.+6.+8.;  // 从128增加到160
            const GRAPH_VIEW_MARGIN: f32 = 32.;   // 从48减少到32
            let graph_columns = ((view_width-GRAPH_VIEW_MARGIN) / GRAPH_WIDTH) as usize;

            let mut draw = true;
            while draw {
                ui.horizontal(|ui| {
                    for _ in 0..graph_columns {
                        if let Some(metric) = metric_iter.next() {
                            // 直接从metrics_data获取最新值，而不是从snapshot.get(metric)
                            let value = {
                                let metrics_data = self.runtime.metrics_service().metrics_data();
                                if let Some(data) = metrics_data.get(metric) {
                                    if let Some(latest) = data.last() {
                                        let val = latest.y;
                                        // 特别关注磁盘读取指标和CPU指标
                                        if *metric == Metric::NodeDiskIoReadBytes || *metric == Metric::NodeDiskIoReadPerSec {
                                            println!("[OVERVIEW] 磁盘读取指标 {} 的最新值: {} (数据点数: {})", metric.as_str(), val, data.len());
                                        }
                                        if *metric == Metric::NodeCpuUsage {
                                            println!("[OVERVIEW] CPU指标 {} 的最新值: {}% (原始数据: {}, 数据点数: {})", metric.as_str(), val, val, data.len());
                                            // 检查格式化后的显示
                                            let formatted = metric.format(val, true, true);
                                            println!("[OVERVIEW] CPU格式化后显示: '{}'", formatted);
                                        }
                                        val
                                    } else {
                                        println!("[OVERVIEW] 警告: metric {} 没有数据点", metric.as_str());
                                        0.0
                                    }
                                } else {
                                    println!("[OVERVIEW] 警告: metric {} 没有找到数据", metric.as_str());
                                    0.0
                                }
                            };
                            self.render_graph(ui,  *metric, value);
                        } else {
                            draw = false;
                        }
                    }
                });
            }
        }

    }

    #[cfg(not(feature = "lean"))]
    fn render_graph(&mut self, ui : &mut Ui, metric : Metric, value : f64) {

        let group = MetricGroup::from(metric);
        let graph_color = group.to_color();

                        let graph_data = {
                    let metrics_data = self.runtime.metrics_service().metrics_data();
                    let data = metrics_data.get(&metric).unwrap();
                    let mut duration = 2 * 60;
                    let available_samples = runtime().metrics_service().samples_since_connection();
                    if available_samples < duration {
                        duration = available_samples;
                    }
                    let len = data.len();
                    let samples = len.min(duration);
                                         let data = data[len-samples..].to_vec();
                    
                    // 为图表数据添加平滑插值，使波形更美观
                    if data.len() > 1 {
                        let mut smoothed_data = Vec::new();
                        for i in 0..data.len() - 1 {
                            smoothed_data.push(data[i]);
                            // 在每两个点之间添加插值点，使线条更平滑
                            let mid_x = (data[i].x + data[i + 1].x) / 2.0;
                            let mid_y = (data[i].y + data[i + 1].y) / 2.0;
                            smoothed_data.push(PlotPoint::new(mid_x, mid_y));
                        }
                        smoothed_data.push(data[data.len() - 1]);
                        smoothed_data
                    } else {
                        data
                    }
                };

        //skip rendering
        if graph_data.len() < 2 {
            return;
        }

        
        ui.vertical(|ui|{
            let frame = 
            Frame::new()
                .stroke(Stroke::new(1.5, theme_color().graph_frame_color))
                .inner_margin(Margin { left: 4, right: 4, top: 5, bottom: 5 })
                .outer_margin(8.)
                .corner_radius(8.)
                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 0))
                ;

            frame.show(ui, |ui| {

                let mut plot = Plot::new(metric.as_str())
                    .legend(Legend::default())
                    .width(128.)
                    .height(32.)
                    .auto_bounds([true, true])
                    .set_margin_fraction(vec2(0.0,0.0) )
                    .show_axes(false)
                    .show_grid(false)
                    .allow_drag([false, false])
                    .allow_scroll(false)
                    .show_background(false)
                    .show_x(false)
                    .show_y(false)
                    .clamp_grid(true) // 确保网格线不会超出边界
                    .include_y(0.0) // 确保包含0点，使填充区域可见
                    ;

                if [Metric::NodeCpuUsage].contains(&metric) {
                    plot = plot.include_y(100.);
                }

                // let color = graph_color.gamma_multiply(0.5);
                
                // 添加渐变填充效果 - 创建封闭的填充区域
                let mut fill_points = graph_data.clone();
                if fill_points.len() > 1 {
                    // 添加底部点以形成封闭区域
                    let first_x = fill_points[0].x;
                    let last_x = fill_points[fill_points.len() - 1].x;
                    let min_y = fill_points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
                    let bottom_y = min_y.min(0.0); // 确保底部至少到0或更低
                    
                    // 在末尾添加底部点，然后回到起点，形成封闭区域
                    fill_points.push(PlotPoint::new(last_x, bottom_y));
                    fill_points.push(PlotPoint::new(first_x, bottom_y));
                }
                
                let fill_line = Line::new("", PlotPoints::Owned(fill_points))
                    .color(graph_color.linear_multiply(0.3)) // 半透明填充
                    .style(LineStyle::Solid)
                    .width(0.0) // 填充线不需要宽度
                    .fill(0.0); // 填充到0基线

                let line = Line::new("", PlotPoints::Owned(graph_data))
                    .color(graph_color)
                    .style(LineStyle::Solid)
                    .width(2.0)
                    .fill(0.0);

                let plot_result = plot.show(ui, |plot_ui| {
                    plot_ui.line(fill_line); // 先绘制填充
                    plot_ui.line(line);      // 再绘制主线条
                });

                let text = format!("{} {}", i18n(metric.title().1).to_uppercase(), metric.format(value, true, true));
                let rich_text_top = RichText::new(&text).size(10.).color(theme_color().raised_text_color);
                let label_top = Label::new(rich_text_top).extend();
                let mut rect_top = plot_result.response.rect;
                rect_top.set_bottom(rect_top.top() + 12.);

                [
                    vec2(-1.0,0.0),vec2(1.0,0.0),vec2(0.0,-1.0),vec2(0.0,1.0),
                    vec2(1.0,1.0),vec2(1.0,-1.0),vec2(-1.0,1.0),vec2(-1.0,-1.0),
                ].iter().for_each(|offset| {
                    let rich_text_back = RichText::new(&text).size(10.).color(theme_color().raised_text_shadow);
                    let label_back = Label::new(rich_text_back).extend();
                    let mut rect_back = rect_top;
                    rect_back.set_center(rect_back.center()+*offset);
                    ui.put(rect_back, label_back);
                });

                ui.put(rect_top, label_top);
            });
        });
    }
}

#[cfg(not(feature = "lean"))]
const METRICS : &[Metric] = &[
    Metric::NodeCpuUsage,
    Metric::NodeResidentSetSizeBytes,
    // Metric::VirtualMemorySizeBytes,
    Metric::NodeFileHandlesCount,
    Metric::NodeDiskIoReadBytes,        // STOR READ (总字节数)
    Metric::NodeDiskIoReadPerSec,      // STOR READ (每秒速度)
    Metric::NodeDiskIoWriteBytes,
    Metric::NodeDiskIoWritePerSec,
    // Metric::BorshLiveConnections,
    // Metric::BorshConnectionAttempts,
    // Metric::BorshHandshakeFailures,
    // Metric::JsonLiveConnections,
    // Metric::JsonConnectionAttempts,
    // Metric::JsonHandshakeFailures,
    Metric::NodeTotalBytesRx,
    Metric::NodeTotalBytesRxPerSecond,
    Metric::NodeTotalBytesTx,
    Metric::NodeTotalBytesTxPerSecond,
    Metric::NodeActivePeers,
    Metric::NodeBlocksSubmittedCount,
    Metric::NodeHeadersProcessedCount,
    Metric::NodeDependenciesProcessedCount,
    Metric::NodeBodiesProcessedCount,
    Metric::NodeTransactionsProcessedCount,
    Metric::NodeChainBlocksProcessedCount,
    Metric::NodeMassProcessedCount,
    Metric::NodeDatabaseBlocksCount,
    Metric::NodeDatabaseHeadersCount,
    Metric::NetworkMempoolSize,
    Metric::NetworkTransactionsPerSecond,
    Metric::NetworkTipHashesCount,
    Metric::NetworkDifficulty,
    Metric::NetworkPastMedianTime,
    Metric::NetworkVirtualParentHashesCount,
    Metric::NetworkVirtualDaaScore,
];