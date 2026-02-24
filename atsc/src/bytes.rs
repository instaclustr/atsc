use crate::error::{Error, Result};

pub(crate) fn take_bytes<'a>(buf: &'a [u8], offset: &mut usize, len: usize) -> Result<&'a [u8]> {
    if *offset > buf.len() || buf.len() - *offset < len {
        return Err(Error::UnexpectedEof {
            offset: *offset as u64,
            expected: len as u64,
        });
    }
    let out = &buf[*offset..*offset + len];
    *offset += len;
    Ok(out)
}

pub(crate) fn take_u8(buf: &[u8], offset: &mut usize) -> Result<u8> {
    Ok(take_bytes(buf, offset, 1)?[0])
}

pub(crate) fn take_u16_le(buf: &[u8], offset: &mut usize) -> Result<u16> {
    let bytes = take_bytes(buf, offset, 2)?;
    let mut arr = [0u8; 2];
    arr.copy_from_slice(bytes);
    Ok(u16::from_le_bytes(arr))
}

pub(crate) fn take_u32_le(buf: &[u8], offset: &mut usize) -> Result<u32> {
    let bytes = take_bytes(buf, offset, 4)?;
    let mut arr = [0u8; 4];
    arr.copy_from_slice(bytes);
    Ok(u32::from_le_bytes(arr))
}

pub(crate) fn take_i64_le(buf: &[u8], offset: &mut usize) -> Result<i64> {
    let bytes = take_bytes(buf, offset, 8)?;
    let mut arr = [0u8; 8];
    arr.copy_from_slice(bytes);
    Ok(i64::from_le_bytes(arr))
}

pub(crate) fn take_f32_le(buf: &[u8], offset: &mut usize) -> Result<f32> {
    let bytes = take_bytes(buf, offset, 4)?;
    let mut arr = [0u8; 4];
    arr.copy_from_slice(bytes);
    Ok(f32::from_le_bytes(arr))
}

pub(crate) fn take_f64_le(buf: &[u8], offset: &mut usize) -> Result<f64> {
    let bytes = take_bytes(buf, offset, 8)?;
    let mut arr = [0u8; 8];
    arr.copy_from_slice(bytes);
    Ok(f64::from_le_bytes(arr))
}

