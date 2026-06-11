//! 从 Buffer 中提取标题大纲(heading outline)的数据结构与方法。

use string_offset::CharOffset;

use super::{
    buffer::Buffer,
    text::{BlockHeaderSize, BlockType, BufferBlockStyle},
};

#[cfg(test)]
#[path = "heading_outline_tests.rs"]
mod tests;

/// 一条标题大纲条目,包含标题级别、文本内容和起始偏移量。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingOutlineEntry {
    /// 标题级别(H1-H6)。
    pub level: BlockHeaderSize,
    /// 标题文本内容(不含 block marker）。
    pub title: String,
    /// block marker 的起始偏移量,用于跳转定位。
    pub start_offset: CharOffset,
}

impl Buffer {
    /// 提取当前 buffer 中所有标题块,返回按文档顺序排列的大纲条目列表。
    pub fn heading_outline(&self) -> Vec<HeadingOutlineEntry> {
        self.outline_blocks()
            .filter_map(|block| {
                if let BlockType::Text(BufferBlockStyle::Header { header_size }) = block.block_type
                {
                    // block.start 是 block marker 字符的偏移,实际文本从 start+1 开始。
                    let text_start = block.start + 1;
                    let title = if text_start < block.end {
                        self.text_in_range(text_start..block.end).to_string()
                    } else {
                        String::new()
                    };
                    Some(HeadingOutlineEntry {
                        level: header_size,
                        title,
                        start_offset: block.start,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
