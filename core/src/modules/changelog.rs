use crate::imports::*;
use crate::egui::easy_mark;
pub struct Changelog {
    #[allow(dead_code)]
    runtime: Runtime,
    changelog : &'static str,
}

impl Changelog {
    pub fn new(runtime: Runtime) -> Self {

        Self { 
            runtime,
            changelog : "# Tondi Dashboard 更新日志\n\n## 版本 1.0.0\n\n### 新功能\n- 多平台支持 (Web, Chrome扩展, 桌面应用)\n- Tondi钱包管理\n- 多账户支持\n- gRPC节点通信\n- 实时区块链数据同步\n\n### 技术特性\n- Rust + WebAssembly架构\n- egui UI框架\n- 加密存储\n- BIP32/BIP39兼容\n\n### 改进\n- 性能优化\n- 安全性增强\n- 用户体验改进\n\n---\n\n*更多详细信息请访问项目仓库*"
        }
    }
}

impl ModuleT for Changelog {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn modal(&self) -> bool {
        true
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        let max_height = ui.available_height() - 64.;

        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(max_height)
                .show(ui, |ui| {
                    easy_mark(ui, self.changelog);
                });

            ui.vertical_centered(|ui|{
                ui.separator();
                ui.add_space(8.);
                if ui.large_button(i18n("Close")).clicked() {
                    core.select::<modules::Overview>();
                }
            });
        });

    }
}
