//! 从 HEIF/AVIF 容器里找出内嵌的 `thmb` 缩略图，并返回一份把 `pitm`
//! 指向缩略图条目的字节副本。
//!
//! 解码器只公开“解码主图”的入口；把主图条目临时改成缩略图条目后，就能
//! 复用同一条解码路径只解缩略图，而不必先解整张照片。容器里没有缩略图
//! 或结构不符合预期时返回 `None`，调用方回退到完整解码。

const META: &[u8; 4] = b"meta";
const PITM: &[u8; 4] = b"pitm";
const IREF: &[u8; 4] = b"iref";
const THMB: &[u8; 4] = b"thmb";

/// 返回一个副本，其 `pitm`（主图条目）被改成 `thmb` 引用指向的缩略图条目。
pub fn with_thumbnail_primary(bytes: &[u8]) -> Option<Vec<u8>> {
    let meta = find_box(bytes, META)?;
    let meta_payload = meta.payload(bytes);
    // meta 是 FullBox：4 字节 version/flags，之后才是子 box。
    let children = meta_payload.get(4..)?;
    let children_start = meta.offset + meta.header + 4;

    let pitm = find_pitm(children)?;
    if pitm.item_id == 0 {
        return None;
    }
    let thumbnail_id = find_thumbnail_reference(children, pitm.item_id)?;
    if thumbnail_id == pitm.item_id {
        return None;
    }

    let mut patched = bytes.to_vec();
    let value_offset = children_start + pitm.offset + pitm.header + 4;
    match pitm.id_width {
        2 => {
            let value = u16::try_from(thumbnail_id).ok()?;
            let start = value_offset;
            patched
                .get_mut(start..start + 2)?
                .copy_from_slice(&value.to_be_bytes());
        }
        4 => {
            let start = value_offset;
            patched
                .get_mut(start..start + 4)?
                .copy_from_slice(&thumbnail_id.to_be_bytes());
        }
        _ => return None,
    }
    Some(patched)
}

struct BoxRef {
    offset: usize,
    size: usize,
    header: usize,
}

impl BoxRef {
    fn payload<'a>(&self, bytes: &'a [u8]) -> &'a [u8] {
        let start = self.offset + self.header;
        let end = self.offset + self.size;
        bytes.get(start..end).unwrap_or(&[])
    }
}

struct Pitm {
    offset: usize,
    header: usize,
    id_width: usize,
    item_id: u32,
}

fn find_box(bytes: &[u8], kind: &[u8; 4]) -> Option<BoxRef> {
    let mut offset = 0usize;
    while offset + 8 <= bytes.len() {
        let size = read_u32(bytes, offset)? as usize;
        let box_kind: [u8; 4] = bytes.get(offset + 4..offset + 8)?.try_into().ok()?;
        let (size, header) = match size {
            0 => (bytes.len() - offset, 8),
            1 => {
                let large = read_u64(bytes, offset + 8)? as usize;
                (large, 16)
            }
            _ => (size, 8),
        };
        if size < header || offset + size > bytes.len() {
            return None;
        }
        if &box_kind == kind {
            return Some(BoxRef {
                offset,
                size,
                header,
            });
        }
        offset += size;
    }
    None
}

fn find_pitm(children: &[u8]) -> Option<Pitm> {
    let b = find_box(children, PITM)?;
    let payload = b.payload(children);
    let version = *payload.first()?;
    let (id_width, item_id) = match version {
        0 => (
            2,
            u32::from(u16::from_be_bytes(payload.get(4..6)?.try_into().ok()?)),
        ),
        1 => (4, read_u32(payload, 4)?),
        _ => return None,
    };
    Some(Pitm {
        offset: b.offset,
        header: b.header,
        id_width,
        item_id,
    })
}

fn find_thumbnail_reference(children: &[u8], primary: u32) -> Option<u32> {
    let b = find_box(children, IREF)?;
    let payload = b.payload(children);
    let version = *payload.first()?;
    if version > 1 {
        return None;
    }
    let id_width = if version == 0 { 2 } else { 4 };
    let mut offset = 4usize;
    while offset + 8 <= payload.len() {
        let entry_size = read_u32(payload, offset)? as usize;
        if entry_size < 8 || offset + entry_size > payload.len() {
            return None;
        }
        let kind: [u8; 4] = payload.get(offset + 4..offset + 8)?.try_into().ok()?;
        let after_type = offset + 8;
        let from = read_item_id(payload, after_type, id_width)?;
        let count_offset = after_type + id_width;
        let count = u16::from_be_bytes(
            payload
                .get(count_offset..count_offset + 2)?
                .try_into()
                .ok()?,
        );
        let mut cursor = count_offset + 2;
        let mut to_items = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            to_items.push(read_item_id(payload, cursor, id_width)?);
            cursor += id_width;
        }
        if &kind == THMB {
            if from == primary {
                if let Some(to) = to_items.first() {
                    return Some(*to);
                }
            }
            if to_items.contains(&primary) {
                return Some(from);
            }
        }
        offset += entry_size;
    }
    None
}

fn read_item_id(bytes: &[u8], offset: usize, width: usize) -> Option<u32> {
    match width {
        2 => Some(u32::from(u16::from_be_bytes(
            bytes.get(offset..offset + 2)?.try_into().ok()?,
        ))),
        4 => read_u32(bytes, offset),
        _ => None,
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_be_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&u32::try_from(8 + payload.len()).unwrap().to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(payload);
        out
    }

    fn meta_with(pitm: &[u8], iref: &[u8]) -> Vec<u8> {
        let mut payload = vec![0, 0, 0, 0];
        payload.extend_from_slice(&boxed(PITM, pitm));
        payload.extend_from_slice(&boxed(IREF, iref));
        let mut out = boxed(b"ftyp", b"heic\0\0\0\0heic");
        out.extend_from_slice(&boxed(META, &payload));
        out
    }

    fn iref_thmb(from: u32, to: u32) -> Vec<u8> {
        let mut entry = Vec::new();
        entry.extend_from_slice(&u16::try_from(from).unwrap().to_be_bytes());
        entry.extend_from_slice(&1u16.to_be_bytes());
        entry.extend_from_slice(&u16::try_from(to).unwrap().to_be_bytes());
        let mut payload = vec![0, 0, 0, 0];
        payload.extend_from_slice(&boxed(THMB, &entry));
        payload
    }

    #[test]
    fn patches_primary_to_thumbnail_item() {
        let bytes = meta_with(&[0, 0, 0, 0, 0, 1], &iref_thmb(2, 1));
        let patched = with_thumbnail_primary(&bytes).expect("thumbnail");
        let meta = find_box(&patched, META).unwrap();
        let children = &meta.payload(&patched)[4..];
        assert_eq!(find_pitm(children).unwrap().item_id, 2);

        // 原字节保持不变。
        let original = find_box(&bytes, META).unwrap();
        let original_children = &original.payload(&bytes)[4..];
        assert_eq!(find_pitm(original_children).unwrap().item_id, 1);
    }

    #[test]
    fn accepts_reference_in_either_direction() {
        let bytes = meta_with(&[0, 0, 0, 0, 0, 1], &iref_thmb(1, 2));
        assert!(with_thumbnail_primary(&bytes).is_some());
    }

    #[test]
    fn ignores_unrelated_references() {
        let mut entry = Vec::new();
        entry.extend_from_slice(&2u16.to_be_bytes());
        entry.extend_from_slice(&1u16.to_be_bytes());
        entry.extend_from_slice(&1u16.to_be_bytes());
        let mut payload = vec![0, 0, 0, 0];
        payload.extend_from_slice(&boxed(b"dimg", &entry));
        let bytes = meta_with(&[0, 0, 0, 0, 0, 1], &payload);
        assert!(with_thumbnail_primary(&bytes).is_none());
    }

    #[test]
    fn malformed_container_is_ignored() {
        assert!(with_thumbnail_primary(b"not a heif file").is_none());
        assert!(with_thumbnail_primary(&[]).is_none());
    }
}
