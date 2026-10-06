use rustpaper_application::{AppError, UserOverrides};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::branding::RgbaImage;
use crate::paths::APP_NAME;
use crate::platform::{platform_error, TrayAction, TrayLabels};

pub struct SystemTray {
    _icon: TrayIcon,
    open: MenuItem,
    settings: MenuItem,
    pause: CheckMenuItem,
    mute: CheckMenuItem,
    quit: MenuItem,
}

impl SystemTray {
    pub fn create(image: &RgbaImage, labels: &TrayLabels) -> Result<Self, AppError> {
        let open = MenuItem::new(&labels.open, true, None);
        let settings = MenuItem::new(&labels.settings, true, None);
        let pause = CheckMenuItem::new(&labels.pause, true, false, None);
        let mute = CheckMenuItem::new(&labels.mute, true, false, None);
        let quit = MenuItem::new(&labels.quit, true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &open,
            &settings,
            &PredefinedMenuItem::separator(),
            &pause,
            &mute,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .map_err(platform_error)?;
        let icon = Icon::from_rgba(image.rgba.clone(), image.width, image.height)
            .map_err(platform_error)?;
        let tray = TrayIconBuilder::new()
            .with_tooltip(APP_NAME)
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .build()
            .map_err(platform_error)?;
        Ok(Self {
            _icon: tray,
            open,
            settings,
            pause,
            mute,
            quit,
        })
    }

    pub fn poll(&self) -> Vec<TrayAction> {
        let mut actions: Vec<TrayAction> = MenuEvent::receiver()
            .try_iter()
            .filter_map(|event| self.menu_action(event.id()))
            .collect();
        let is_left_clicked = TrayIconEvent::receiver().try_iter().any(|event| {
            matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            )
        });
        if is_left_clicked {
            actions.push(TrayAction::OpenStudio);
        }
        actions
    }

    pub fn relabel(&self, labels: &TrayLabels) {
        self.open.set_text(&labels.open);
        self.settings.set_text(&labels.settings);
        self.pause.set_text(&labels.pause);
        self.mute.set_text(&labels.mute);
        self.quit.set_text(&labels.quit);
    }

    pub fn reflect(&self, overrides: UserOverrides) {
        self.pause.set_checked(overrides.is_paused);
        self.mute.set_checked(overrides.is_muted);
    }

    fn menu_action(&self, id: &tray_icon::menu::MenuId) -> Option<TrayAction> {
        if id == self.open.id() {
            return Some(TrayAction::OpenStudio);
        }
        if id == self.settings.id() {
            return Some(TrayAction::OpenSettings);
        }
        if id == self.pause.id() {
            return Some(TrayAction::SetPaused(self.pause.is_checked()));
        }
        if id == self.mute.id() {
            return Some(TrayAction::SetMuted(self.mute.is_checked()));
        }
        (id == self.quit.id()).then_some(TrayAction::Quit)
    }
}
