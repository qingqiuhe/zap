//! Markdown 大纲面板 — 左侧 Tool Panel,展示当前文档的标题层级列表。
//!
//! 点击某条标题时,emit `JumpToOffset` 事件通知外部跳转到对应偏移位置。

use string_offset::CharOffset;
use warp_core::ui::appearance::Appearance;
use warpui::elements::{
    ClippedScrollStateHandle, ClippedScrollable, Container, CrossAxisAlignment, Element, Fill,
    Flex, Hoverable, MainAxisSize, MouseStateHandle, ParentElement, ScrollbarWidth, Text,
};
use warpui::platform::Cursor;
use warpui::{AppContext, Entity, TypedActionView, View, ViewContext};

use warp_editor::content::heading_outline::HeadingOutlineEntry;
use warp_editor::content::text::BlockHeaderSize;

// ---- 视觉常量 ----
/// 每级标题的左缩进像素。
const HEADING_INDENT_PX: f32 = 16.0;
/// 行垂直内边距。
const ROW_PADDING_VERTICAL: f32 = 4.0;
/// 行水平内边距。
const ROW_PADDING_HORIZONTAL: f32 = 8.0;

/// 面板动作:用户在面板上的交互。
#[derive(Clone, Debug)]
pub enum MarkdownOutlinePanelAction {
    /// 用户点击了某条标题,请求跳转到对应偏移。
    JumpToHeading { offset: CharOffset },
    /// 折叠/展开所有标题(保留扩展空间)。
    ToggleAll,
}

/// 面板事件:面板对外发出的通知。
#[derive(Clone, Debug)]
pub enum MarkdownOutlinePanelEvent {
    /// 请求编辑器滚动到指定偏移位置。
    JumpToOffset { offset: CharOffset },
}

/// Markdown 大纲面板视图。
pub struct MarkdownOutlinePanel {
    /// 当前文档的标题条目列表。
    entries: Vec<HeadingOutlineEntry>,
    /// 当前选中(高亮)的条目索引。
    #[allow(dead_code)]
    active_index: Option<usize>,
    /// 滚动状态。
    scroll_state: ClippedScrollStateHandle,
    /// 每行的鼠标悬停状态(按索引缓存)。
    row_states: Vec<MouseStateHandle>,
}

impl MarkdownOutlinePanel {
    pub fn new(_ctx: &mut ViewContext<Self>) -> Self {
        Self {
            entries: Vec::new(),
            active_index: None,
            scroll_state: ClippedScrollStateHandle::default(),
            row_states: Vec::new(),
        }
    }

    /// 更新标题条目列表;外部在文件加载/切换时调用。
    pub fn update_entries(
        &mut self,
        entries: Vec<HeadingOutlineEntry>,
        ctx: &mut ViewContext<Self>,
    ) {
        self.row_states
            .resize_with(entries.len(), MouseStateHandle::default);
        self.entries = entries;
        self.active_index = None;
        ctx.notify();
    }

    /// 渲染单行标题条目。
    fn render_row(
        &self,
        index: usize,
        entry: &HeadingOutlineEntry,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let text_color = theme.main_text_color(theme.background());
        let level_usize: usize = entry.level.into();
        let indent = (level_usize.saturating_sub(1)) as f32 * HEADING_INDENT_PX;

        let label = Text::new_inline(
            &entry.title,
            appearance.ui_font_family(),
            appearance.ui_font_subheading(),
        )
        .with_color(text_color.into())
        .finish();

        let row_content = Container::new(label)
            .with_padding_top(ROW_PADDING_VERTICAL)
            .with_padding_bottom(ROW_PADDING_VERTICAL)
            .with_padding_left(ROW_PADDING_HORIZONTAL + indent)
            .with_padding_right(ROW_PADDING_HORIZONTAL)
            .finish();

        let offset = entry.start_offset;
        let state = self
            .row_states
            .get(index)
            .cloned()
            .unwrap_or_default();

        Hoverable::new(state, move |_| row_content)
            .with_cursor(Cursor::PointingHand)
            .on_click(move |ctx, _, _| {
                ctx.dispatch_typed_action(MarkdownOutlinePanelAction::JumpToHeading { offset });
            })
            .finish()
    }

    /// 渲染空状态提示。
    fn render_empty_state(&self, appearance: &Appearance) -> Box<dyn Element> {
        let theme = appearance.theme();
        let muted = theme.sub_text_color(theme.background());

        Container::new(
            Text::new_inline(
                "Open a Markdown file to see outline",
                appearance.ui_font_family(),
                appearance.ui_font_subheading(),
            )
            .with_color(muted.into())
            .finish(),
        )
        .with_padding_top(20.0)
        .with_padding_bottom(20.0)
        .with_padding_left(ROW_PADDING_HORIZONTAL)
        .with_padding_right(ROW_PADDING_HORIZONTAL)
        .finish()
    }
}

impl Entity for MarkdownOutlinePanel {
    type Event = MarkdownOutlinePanelEvent;
}

impl TypedActionView for MarkdownOutlinePanel {
    type Action = MarkdownOutlinePanelAction;

    fn handle_action(
        &mut self,
        action: &MarkdownOutlinePanelAction,
        ctx: &mut ViewContext<Self>,
    ) {
        match action {
            MarkdownOutlinePanelAction::JumpToHeading { offset } => {
                ctx.emit(MarkdownOutlinePanelEvent::JumpToOffset { offset: *offset });
            }
            MarkdownOutlinePanelAction::ToggleAll => {
                // 保留扩展空间,当前为空操作。
            }
        }
    }
}

impl View for MarkdownOutlinePanel {
    fn ui_name() -> &'static str {
        "MarkdownOutlinePanel"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);

        if self.entries.is_empty() {
            return self.render_empty_state(appearance);
        }

        let mut col = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_main_axis_size(MainAxisSize::Min);

        for (i, entry) in self.entries.iter().enumerate() {
            col.add_child(self.render_row(i, entry, appearance));
        }

        let theme = appearance.theme();
        let scrollbar_color = theme.disabled_text_color(theme.background()).into();
        let scrollbar_thumb_hover = theme.main_text_color(theme.background()).into();

        ClippedScrollable::vertical(
            self.scroll_state.clone(),
            col.finish(),
            ScrollbarWidth::Auto,
            scrollbar_color,
            scrollbar_thumb_hover,
            Fill::None,
        )
        .finish()
    }
}
