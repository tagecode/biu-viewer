/// 用户在查看时叠加的旋转和翻转。EXIF 校正已经写入像素，重置这里不会把它撤销。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransformState {
    /// 顺时针 90° 的次数，取值 0-3。
    pub quarter_turns_cw: u8,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl TransformState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn is_identity(self) -> bool {
        self == Self::default()
    }

    pub fn rotate_cw(&mut self) {
        self.quarter_turns_cw = (self.quarter_turns_cw + 1) % 4;
    }

    pub fn rotate_ccw(&mut self) {
        self.quarter_turns_cw = (self.quarter_turns_cw + 3) % 4;
    }

    pub fn flip_horizontal(&mut self) {
        self.flip_horizontal = !self.flip_horizontal;
    }

    pub fn flip_vertical(&mut self) {
        self.flip_vertical = !self.flip_vertical;
    }

    /// 90°/270° 旋转会交换宽高。
    pub fn swaps_dimensions(self) -> bool {
        self.quarter_turns_cw % 2 == 1
    }

    /// 变换后的显示尺寸。
    pub fn display_size(self, width: u32, height: u32) -> (u32, u32) {
        if self.swaps_dimensions() {
            (height, width)
        } else {
            (width, height)
        }
    }

    /// 把变换应用到 RGBA8 像素。顺序固定为先旋转，再水平翻转，最后垂直翻转。
    /// 缓冲区长度与尺寸不匹配时返回 `None`。
    pub fn apply_to_rgba(
        self,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Option<(u32, u32, Vec<u8>)> {
        let expected = usize::try_from(u64::from(width) * u64::from(height) * 4).ok()?;
        if rgba.len() != expected {
            return None;
        }

        let mut buffer = rgba.to_vec();
        let (mut width, mut height) = (width, height);
        for _ in 0..self.quarter_turns_cw {
            buffer = rotate_cw(width, height, &buffer);
            std::mem::swap(&mut width, &mut height);
        }
        if self.flip_horizontal {
            buffer = flip_horizontal(width, height, &buffer);
        }
        if self.flip_vertical {
            buffer = flip_vertical(width, height, &buffer);
        }
        Some((width, height, buffer))
    }
}

fn rotate_cw(width: u32, height: u32, source: &[u8]) -> Vec<u8> {
    let mut output = vec![0_u8; source.len()];
    for y in 0..height {
        for x in 0..width {
            let source_index = ((y * width + x) * 4) as usize;
            let target_x = height - 1 - y;
            let target_y = x;
            let target_index = ((target_y * height + target_x) * 4) as usize;
            output[target_index..target_index + 4]
                .copy_from_slice(&source[source_index..source_index + 4]);
        }
    }
    output
}

fn flip_horizontal(width: u32, height: u32, source: &[u8]) -> Vec<u8> {
    let mut output = vec![0_u8; source.len()];
    for y in 0..height {
        for x in 0..width {
            let source_index = ((y * width + x) * 4) as usize;
            let target_index = ((y * width + (width - 1 - x)) * 4) as usize;
            output[target_index..target_index + 4]
                .copy_from_slice(&source[source_index..source_index + 4]);
        }
    }
    output
}

fn flip_vertical(width: u32, height: u32, source: &[u8]) -> Vec<u8> {
    let mut output = vec![0_u8; source.len()];
    for y in 0..height {
        for x in 0..width {
            let source_index = ((y * width + x) * 4) as usize;
            let target_index = (((height - 1 - y) * width + x) * 4) as usize;
            output[target_index..target_index + 4]
                .copy_from_slice(&source[source_index..source_index + 4]);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: [u8; 4] = [1, 0, 0, 255];
    const B: [u8; 4] = [0, 1, 0, 255];
    const C: [u8; 4] = [0, 0, 1, 255];
    const D: [u8; 4] = [1, 1, 1, 255];

    fn pattern() -> Vec<u8> {
        [A, B, C, D].concat()
    }

    fn pixels(rgba: &[u8]) -> Vec<[u8; 4]> {
        rgba.as_chunks::<4>().0.to_vec()
    }

    #[test]
    fn reset_returns_to_identity() {
        let mut transform = TransformState {
            quarter_turns_cw: 3,
            flip_horizontal: true,
            flip_vertical: true,
        };
        transform.reset();
        assert!(transform.is_identity());
    }

    #[test]
    fn four_clockwise_turns_return_to_identity() {
        let mut transform = TransformState::default();
        for _ in 0..4 {
            transform.rotate_cw();
        }
        assert!(transform.is_identity());
    }

    #[test]
    fn opposite_rotations_cancel_out() {
        let mut transform = TransformState::default();
        transform.rotate_cw();
        transform.rotate_ccw();
        assert!(transform.is_identity());
    }

    #[test]
    fn flipping_twice_in_the_same_direction_returns_to_identity() {
        let mut transform = TransformState {
            flip_horizontal: true,
            ..Default::default()
        };
        transform.flip_horizontal();
        assert!(transform.is_identity());

        let mut transform = TransformState {
            flip_vertical: true,
            ..Default::default()
        };
        transform.flip_vertical();
        assert!(transform.is_identity());
    }

    #[test]
    fn rotate_swaps_dimensions_only_for_odd_turns() {
        let mut transform = TransformState::default();
        assert_eq!(transform.display_size(2, 3), (2, 3));
        transform.rotate_cw();
        assert_eq!(transform.display_size(2, 3), (3, 2));
        transform.rotate_cw();
        assert_eq!(transform.display_size(2, 3), (2, 3));
    }

    #[test]
    fn rotates_pixels_clockwise() {
        let (width, height, rgba) = TransformState {
            quarter_turns_cw: 1,
            ..Default::default()
        }
        .apply_to_rgba(2, 2, &pattern())
        .unwrap();
        assert_eq!((width, height), (2, 2));
        assert_eq!(pixels(&rgba), vec![C, A, D, B]);
    }

    #[test]
    fn flips_pixels_after_rotation() {
        let (width, height, rgba) = TransformState {
            quarter_turns_cw: 1,
            flip_horizontal: true,
            flip_vertical: false,
        }
        .apply_to_rgba(2, 2, &pattern())
        .unwrap();
        assert_eq!((width, height), (2, 2));
        assert_eq!(pixels(&rgba), vec![A, C, B, D]);

        let (_, _, rgba) = TransformState {
            flip_vertical: true,
            ..Default::default()
        }
        .apply_to_rgba(2, 2, &pattern())
        .unwrap();
        assert_eq!(pixels(&rgba), vec![C, D, A, B]);
    }

    #[test]
    fn rotate_ninety_degrees_changes_layout() {
        // 2x1: A B -> 顺时针旋转后 1x2: A / B
        let (width, height, rgba) = TransformState {
            quarter_turns_cw: 1,
            ..Default::default()
        }
        .apply_to_rgba(2, 1, &[A, B].concat())
        .unwrap();
        assert_eq!((width, height), (1, 2));
        assert_eq!(pixels(&rgba), vec![A, B]);
    }

    #[test]
    fn mismatched_buffer_is_rejected() {
        assert!(TransformState::default()
            .apply_to_rgba(2, 2, &[0, 0, 0, 0])
            .is_none());
    }
}
