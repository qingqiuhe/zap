use crate::content::{
    buffer::Buffer,
    selection_model::BufferSelectionModel,
    text::{BlockHeaderSize, BufferBlockStyle, IndentBehavior, TextStyles},
};
use string_offset::CharOffset;
use warpui::App;

#[test]
fn test_heading_outline_empty_buffer() {
    App::test((), |mut app| async move {
        let buffer = app.add_model(|_| Buffer::new(Box::new(|_, _| IndentBehavior::Ignore)));

        buffer.update(&mut app, |buffer, _ctx| {
            let outline = buffer.heading_outline();
            assert!(outline.is_empty());
        });
    });
}

#[test]
fn test_heading_outline_single_h1() {
    App::test((), |mut app| async move {
        let buffer = app.add_model(|_| Buffer::new(Box::new(|_, _| IndentBehavior::Ignore)));
        let selection = app.add_model(|_| BufferSelectionModel::new(buffer.clone()));

        buffer.update(&mut app, |buffer, ctx| {
            let _ = buffer.edit_internal_first_selection(
                CharOffset::from(1)..CharOffset::from(1),
                "Hello World",
                TextStyles::default(),
                selection.clone(),
                ctx,
            );
            // 将整行设置为 H1 标题
            buffer.block_style_range(
                CharOffset::from(1)..CharOffset::from(12),
                BufferBlockStyle::Header {
                    header_size: BlockHeaderSize::Header1,
                },
                selection.clone(),
                ctx,
            );

            let outline = buffer.heading_outline();
            assert_eq!(outline.len(), 1);
            assert_eq!(outline[0].level, BlockHeaderSize::Header1);
            assert_eq!(outline[0].title, "Hello World");
        });
    });
}

#[test]
fn test_heading_outline_multiple_levels() {
    App::test((), |mut app| async move {
        let buffer = app.add_model(|_| Buffer::new(Box::new(|_, _| IndentBehavior::Ignore)));
        let selection = app.add_model(|_| BufferSelectionModel::new(buffer.clone()));

        buffer.update(&mut app, |buffer, ctx| {
            // 插入文本: "TitleSubtitleSection"
            let _ = buffer.edit_internal_first_selection(
                CharOffset::from(1)..CharOffset::from(1),
                "TitleSubtitleSection",
                TextStyles::default(),
                selection.clone(),
                ctx,
            );

            // 按从后向前设置块样式,避免偏移量变化
            // "Section" 从 offset 14 开始(1-based: char 14..21)
            buffer.block_style_range(
                CharOffset::from(14)..CharOffset::from(21),
                BufferBlockStyle::Header {
                    header_size: BlockHeaderSize::Header3,
                },
                selection.clone(),
                ctx,
            );
            // "Subtitle" 从 offset 6 开始(1-based: char 6..14)
            buffer.block_style_range(
                CharOffset::from(6)..CharOffset::from(14),
                BufferBlockStyle::Header {
                    header_size: BlockHeaderSize::Header2,
                },
                selection.clone(),
                ctx,
            );
            // "Title" 从 offset 1 开始(1-based: char 1..6)
            buffer.block_style_range(
                CharOffset::from(1)..CharOffset::from(6),
                BufferBlockStyle::Header {
                    header_size: BlockHeaderSize::Header1,
                },
                selection.clone(),
                ctx,
            );

            let outline = buffer.heading_outline();
            assert_eq!(outline.len(), 3);

            assert_eq!(outline[0].level, BlockHeaderSize::Header1);
            assert_eq!(outline[0].title, "Title");

            assert_eq!(outline[1].level, BlockHeaderSize::Header2);
            assert_eq!(outline[1].title, "Subtitle");

            assert_eq!(outline[2].level, BlockHeaderSize::Header3);
            assert_eq!(outline[2].title, "Section");
        });
    });
}

#[test]
fn test_heading_outline_ignores_non_heading_blocks() {
    App::test((), |mut app| async move {
        let buffer = app.add_model(|_| Buffer::new(Box::new(|_, _| IndentBehavior::Ignore)));
        let selection = app.add_model(|_| BufferSelectionModel::new(buffer.clone()));

        buffer.update(&mut app, |buffer, ctx| {
            let _ = buffer.edit_internal_first_selection(
                CharOffset::from(1)..CharOffset::from(1),
                "HeadingCodeBlock",
                TextStyles::default(),
                selection.clone(),
                ctx,
            );

            // 后半部分设为 code block
            buffer.block_style_range(
                CharOffset::from(8)..CharOffset::from(17),
                BufferBlockStyle::CodeBlock {
                    code_block_type: Default::default(),
                },
                selection.clone(),
                ctx,
            );
            // 前半部分设为 H2 标题
            buffer.block_style_range(
                CharOffset::from(1)..CharOffset::from(8),
                BufferBlockStyle::Header {
                    header_size: BlockHeaderSize::Header2,
                },
                selection.clone(),
                ctx,
            );

            let outline = buffer.heading_outline();
            // 只有标题块应该被提取
            assert_eq!(outline.len(), 1);
            assert_eq!(outline[0].level, BlockHeaderSize::Header2);
            assert_eq!(outline[0].title, "Heading");
        });
    });
}
