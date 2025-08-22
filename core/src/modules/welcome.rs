use crate::imports::*;

pub struct Welcome {
    #[allow(dead_code)]
    runtime: Runtime,
    settings : Settings,
    grpc_network_interface : NetworkInterfaceEditor,
}

impl Welcome {
    pub fn new(runtime: Runtime) -> Self {

        #[allow(unused_mut)]
        let mut settings = Settings::default();

        #[cfg(target_arch = "wasm32")] {
            settings.node.node_kind = TondidNodeKind::Remote;
        }

        Self { 
            runtime, 
            grpc_network_interface: NetworkInterfaceEditor::from(&settings.node.grpc_network_interface),
            settings,
        }
    }

    pub fn render_native(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {

        let mut error = None;

        ui.heading(i18n("Welcome to Tondi Dashboard"));
        ui.add_space(16.0);
        ui.label(i18n("Please configure your Tondi Dashboard settings"));
        ui.add_space(16.0);

        CollapsingHeader::new(i18n("Settings"))
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new(i18n("Tondi Network"))
                    .default_open(true)
                    .show(ui, |ui| {

                            ui.horizontal_wrapped(|ui| {
                                Network::iter().for_each(|network| {
                                    ui.radio_value(&mut self.settings.node.network, *network, format!("{} ({})",network.name(),network.describe()));

                                });
                            });

                            match self.settings.node.network {
                                Network::Mainnet => {
                                    // ui.colored_label(theme_color().warning_color, i18n("Please note that this is a beta release. Until this message is removed, please avoid using the wallet with mainnet funds."));
                                }
                                Network::Testnet => { }
                                Network::Devnet => { }
                            }
                        });
                
                CollapsingHeader::new(i18n("Tondi p2p Node & Connection"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            // TondidNodeKind::iter().for_each(|node| {
                            [
                                TondidNodeKind::Disable,
                                TondidNodeKind::Remote,
                                #[cfg(not(target_arch = "wasm32"))]
                                TondidNodeKind::IntegratedAsDaemon,
                                // TondidNodeKind::ExternalAsDaemon,
                                // TondidNodeKind::IntegratedInProc,
                            ].iter().for_each(|node_kind| {
                                ui.radio_value(&mut self.settings.node.node_kind, *node_kind, node_kind.to_string()).on_hover_text_at_pointer(node_kind.describe());
                            });
                        });

                        if self.settings.node.node_kind == TondidNodeKind::Remote {
                            error = crate::modules::settings::Settings::render_remote_settings(core,ui,&mut self.settings.node, &mut self.grpc_network_interface);
                        }
                    });

                CollapsingHeader::new(i18n("User Interface"))
                    .default_open(true)
                    .show(ui, |ui| {

                        ui.horizontal(|ui| {

                            ui.label(i18n("Language:"));

                            let language_code = core.settings.language_code.clone();
                            let dictionary = i18n::dictionary();
                            let language = dictionary.language_title(language_code.as_str()).unwrap();//.unwrap();
                            egui::ComboBox::from_id_salt("language_selector")
                                .selected_text(language)
                                .width(150.0)  // 增加ComboBox按钮宽度
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(150.0);  // 增加下拉面板最小宽度
                                    dictionary.enabled_languages().into_iter().for_each(|(code,lang)| {
                                        if ui.selectable_value(&mut self.settings.language_code, code.to_string(), lang).clicked() {
                                            // 立即激活新选择的语言
                                            if let Err(err) = dictionary.activate_language_code(code) {
                                                log::error!("Unable to activate language {}: {}", code, err);
                                            }
                                        }
                                    });
                                });

                            ui.add_space(16.);
                            ui.label(i18n("Theme Color:"));

                            let mut theme_color = self.settings.user_interface.theme_color.clone();
                            egui::ComboBox::from_id_salt("theme_color_selector")
                                .selected_text(theme_color.as_str())
                                .width(150.0)  // 增加ComboBox按钮宽度
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(150.0);  // 增加下拉面板最小宽度
                                    theme_colors().keys().for_each(|name| {
                                        ui.selectable_value(&mut theme_color, name.to_string(), name);
                                    });
                                });
                                
                            if theme_color != self.settings.user_interface.theme_color {
                                self.settings.user_interface.theme_color = theme_color;
                                apply_theme_color_by_name(ui.ctx(), self.settings.user_interface.theme_color.clone());
                            }

                            ui.add_space(16.);
                            ui.label(i18n("Theme Style:"));

                            let mut theme_style = self.settings.user_interface.theme_style.clone();
                            egui::ComboBox::from_id_salt("theme_style_selector")
                                .selected_text(theme_style.as_str())
                                .width(150.0)  // 增加ComboBox按钮宽度
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(150.0);  // 增加下拉面板最小宽度
                                    theme_styles().keys().for_each(|name| {
                                        ui.selectable_value(&mut theme_style, name.to_string(), name);
                                    });
                                });
                                
                            if theme_style != self.settings.user_interface.theme_style {
                                self.settings.user_interface.theme_style = theme_style;
                                apply_theme_style_by_name(ui.ctx(), self.settings.user_interface.theme_style.clone());
                            }
                        });        
                    });

                ui.add_space(32.0);
                if let Some(error) = error {
                    ui.vertical_centered(|ui| {
                        ui.colored_label(theme_color().alert_color, error);
                    });
                    ui.add_space(32.0);
                } else {
                    
                    ui.horizontal(|ui| {
                        ui.add_space(
                            ui.available_width()
                                - 16.
                                - (theme_style().medium_button_size.x + ui.spacing().item_spacing.x),
                        );
                        if ui.medium_button(format!("{} {}", egui_phosphor::light::CHECK, i18n("Apply"))).clicked() {
                            let mut settings = self.settings.clone();
                            settings.initialized = true;
                            
                            // 确保语言设置被正确应用
                            if let Err(err) = i18n::dictionary().activate_language_code(&settings.language_code) {
                                log::error!("Unable to activate language {}: {}", settings.language_code, err);
                            }
                            
                            let message = i18n("Unable to store settings");
                            settings.store_sync().expect(message);
                            self.runtime.tondi_service().update_services(&self.settings.node, None);
                            core.settings = settings.clone();
                            core.get_mut::<modules::Settings>().load(settings);
                            cfg_if!{
                                if #[cfg(not(target_arch = "wasm32"))] {
                                    core.select::<modules::Changelog>();
                                } else {
                                    core.select::<modules::Overview>();
                                }
                            }
                        }
                    });
                }

                ui.separator();
        });
        
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            // ui.colored_label(theme_color().alert_color, "Please note - this is a beta release - Tondi Dashboard is still in early development and is not yet ready for production use.");
            // ui.add_space(32.0);
            ui.label(format!("Tondi Dashboard v{}  •  Tondi Client v{}", env!("CARGO_PKG_VERSION"), tondi_wallet_core::version()));
            ui.hyperlink_to(
                "https://tondi.org",
                "https://tondi.org",
            );
    
        });
    }

    pub fn render_web(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {
        let mut proceed = false;

        Panel::new(self)
            .with_caption(i18n("Welcome to Tondi Dashboard"))
            .with_header(|_this, ui| {
                ui.label(i18n("Please select Tondi network"));
            })
            .with_body(|this, ui| {
                Network::iter().for_each(|network| {
                    if ui.add_sized(
                            theme_style().large_button_size,
                            CompositeButton::opt_image_and_text(
                                None,
                                Some(network.name().into()),
                                Some(network.describe().into()),
                            ),
                        )
                        .clicked()
                    {
                        this.settings.node.network = *network;
                        // 自动更新端口配置
                        this.settings.node.update_ports_for_network();
                        proceed = true;
                    }

                    ui.add_space(8.);
                });

                // ui.add_space(32.0);
                // ui.colored_label(theme_color().alert_color, RichText::new("β").size(64.0));
                // ui.add_space(8.0);
                // ui.colored_label(theme_color().alert_color, "Please note - this is a beta release - Tondi Dashboard is still in early development and is not yet ready for production use.");
            })
            .render(ui);        

        if proceed {
            let mut settings = self.settings.clone();
            settings.initialized = true;
            let message = i18n("Unable to store settings");
            settings.store_sync().expect(message);
            core.settings = settings.clone();
            self.runtime.tondi_service().update_services(&settings.node, None);

            core.get_mut::<modules::Settings>().load(settings);
            core.select::<modules::Overview>();
        }

    }

}

impl ModuleT for Welcome {

    fn style(&self) -> ModuleStyle {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                ModuleStyle::Mobile
            } else {
                ModuleStyle::Default
            }
        }
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                self.render_native(core, ui)
            } else {
                self.render_web(core, ui)
            }
        }
    }

}
