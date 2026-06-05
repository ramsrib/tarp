use warpui::elements::{
    CacheOption, ConstrainedBox, Container, CrossAxisAlignment, Element, Flex, Image,
    MainAxisAlignment, MainAxisSize, MouseStateHandle, ParentElement, Text, Wrap,
};
use warpui::assets::asset_cache::AssetSource;
use warp_core::ui::theme::color::internal_colors;
use warpui::ui_components::components::UiComponent;
use warpui::{AppContext, Entity, View, ViewContext, ViewHandle};

use super::settings_page::{
    MatchData, PageType, SettingsPageEvent, SettingsPageMeta, SettingsPageViewHandle,
    SettingsWidget,
};
use super::SettingsSection;
use crate::appearance::Appearance;
use crate::channel::ChannelState;
use crate::workspace::WorkspaceAction;

pub struct AboutPageView {
    page: PageType<Self>,
}

impl AboutPageView {
    pub fn new(_ctx: &mut ViewContext<AboutPageView>) -> Self {
        AboutPageView {
            page: PageType::new_monolith(AboutPageWidget::default(), None, false),
        }
    }
}

impl Entity for AboutPageView {
    type Event = SettingsPageEvent;
}

impl View for AboutPageView {
    fn ui_name() -> &'static str {
        "AboutPage"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

#[derive(Default)]
struct AboutPageWidget {
    copy_version_button_mouse_state: MouseStateHandle,
}

impl SettingsWidget for AboutPageWidget {
    type View = AboutPageView;

    fn search_terms(&self) -> &str {
        "about tarp version"
    }

    fn render(
        &self,
        _view: &AboutPageView,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let ui_builder = appearance.ui_builder();

        let version = ChannelState::app_version().unwrap_or("v#.##.###");

        let version_text = ui_builder
            .span(version.to_string())
            .with_soft_wrap()
            .build()
            .with_margin_top(16.)
            .finish();

        let copy_version_icon = appearance
            .ui_builder()
            .copy_button(16., self.copy_version_button_mouse_state.clone())
            .build()
            .on_click(move |ctx, _, _| {
                ctx.dispatch_typed_action(WorkspaceAction::CopyVersion(version));
            })
            .finish();

        let version_row = Wrap::row()
            .with_main_axis_alignment(MainAxisAlignment::Center)
            .with_children([
                version_text,
                Container::new(copy_version_icon)
                    .with_margin_top(16.)
                    .with_padding_left(6.)
                    .finish(),
            ]);

        // Tarp logo (the app icon), centered above the name.
        let logo = ConstrainedBox::new(
            Image::new(
                AssetSource::Bundled {
                    path: "bundled/svg/tarp-logo.png",
                },
                CacheOption::BySize,
            )
            .finish(),
        )
        .with_max_height(96.)
        .with_max_width(96.)
        .finish();

        let name_and_version = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(logo)
            .with_child(
                Container::new(ui_builder.span("Tarp").build().finish())
                    .with_margin_top(16.)
                    .finish(),
            )
            .with_child(version_row.finish())
            .finish();

        // Subtle legal footer pinned to the bottom. The "portions © … Denver
        // Technologies, Inc." notice is REQUIRED by AGPL/MIT and must remain
        // (see docs/REMOVED.md).
        let legal_footer = Container::new(
            ConstrainedBox::new(
                Text::new(
                    "Tarp — a fork of Warp. AGPL-3.0 / MIT. \
                     © 2026 Tarp Project; \
                     portions © 2020-2026 Denver Technologies, Inc."
                        .to_string(),
                    appearance.ui_font_family(),
                    appearance.ui_font_size() - 2.0,
                )
                .with_color(internal_colors::fg_overlay_6(theme).into())
                .finish(),
            )
            .with_max_width(520.)
            .finish(),
        )
        .with_margin_bottom(14.)
        .finish();

        // Full-height column: empty top spacer + centered name/version + footer.
        // SpaceBetween pushes the footer to the bottom while keeping the name
        // roughly centered above it.
        Flex::column()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(Flex::column().finish())
            .with_child(name_and_version)
            .with_child(legal_footer)
            .finish()
    }
}

impl SettingsPageMeta for AboutPageView {
    fn section() -> SettingsSection {
        SettingsSection::About
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        true
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<AboutPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<AboutPageView>) -> Self {
        SettingsPageViewHandle::About(view_handle)
    }
}
