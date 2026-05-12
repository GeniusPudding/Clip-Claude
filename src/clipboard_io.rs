use anyhow::{Context, Result};
use arboard::ImageData;

#[cfg(windows)]
const CF_DIBV5: u32 = 17;
#[cfg(windows)]
const CF_UNICODETEXT: u32 = 13;
#[cfg(windows)]
const BI_BITFIELDS: u32 = 3;
#[cfg(windows)]
const LCS_WINDOWS_COLOR_SPACE: u32 = 0x5769_6E20; // 'Win '
#[cfg(windows)]
const LCS_GM_IMAGES: u32 = 4;

#[cfg(windows)]
pub fn write_image_and_text(img: ImageData, text: &str) -> Result<()> {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };

    let width = img.width as i32;
    let height = img.height as i32;
    let mut bgra = img.bytes.into_owned();
    for chunk in bgra.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
    let dib = build_dibv5(width, height, &bgra);

    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let mut utf16_bytes: Vec<u8> = Vec::with_capacity(utf16.len() * 2);
    for c in &utf16 {
        utf16_bytes.extend_from_slice(&c.to_le_bytes());
    }

    unsafe {
        let dib_handle = global_alloc_and_copy(&dib)?;
        let text_handle = global_alloc_and_copy(&utf16_bytes)?;

        OpenClipboard(None).context("OpenClipboard")?;
        EmptyClipboard().context("EmptyClipboard")?;
        SetClipboardData(CF_DIBV5, Some(HANDLE(dib_handle.0)))
            .context("SetClipboardData(CF_DIBV5)")?;
        SetClipboardData(CF_UNICODETEXT, Some(HANDLE(text_handle.0)))
            .context("SetClipboardData(CF_UNICODETEXT)")?;
        CloseClipboard().context("CloseClipboard")?;
    }
    Ok(())
}

#[cfg(windows)]
fn build_dibv5(width: i32, height: i32, bgra: &[u8]) -> Vec<u8> {
    let pixel_size = bgra.len() as u32;
    let mut buf = Vec::with_capacity(124 + bgra.len());
    buf.extend_from_slice(&124u32.to_le_bytes());                    // bV5Size
    buf.extend_from_slice(&width.to_le_bytes());                     // bV5Width
    buf.extend_from_slice(&(-height).to_le_bytes());                 // bV5Height (negative = top-down)
    buf.extend_from_slice(&1u16.to_le_bytes());                      // bV5Planes
    buf.extend_from_slice(&32u16.to_le_bytes());                     // bV5BitCount
    buf.extend_from_slice(&BI_BITFIELDS.to_le_bytes());              // bV5Compression
    buf.extend_from_slice(&pixel_size.to_le_bytes());                // bV5SizeImage
    buf.extend_from_slice(&0i32.to_le_bytes());                      // bV5XPelsPerMeter
    buf.extend_from_slice(&0i32.to_le_bytes());                      // bV5YPelsPerMeter
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5ClrUsed
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5ClrImportant
    buf.extend_from_slice(&0x00FF_0000u32.to_le_bytes());            // bV5RedMask
    buf.extend_from_slice(&0x0000_FF00u32.to_le_bytes());            // bV5GreenMask
    buf.extend_from_slice(&0x0000_00FFu32.to_le_bytes());            // bV5BlueMask
    buf.extend_from_slice(&0xFF00_0000u32.to_le_bytes());            // bV5AlphaMask
    buf.extend_from_slice(&LCS_WINDOWS_COLOR_SPACE.to_le_bytes());   // bV5CSType
    buf.extend_from_slice(&[0u8; 36]);                               // bV5Endpoints
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5GammaRed
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5GammaGreen
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5GammaBlue
    buf.extend_from_slice(&LCS_GM_IMAGES.to_le_bytes());             // bV5Intent
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5ProfileData
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5ProfileSize
    buf.extend_from_slice(&0u32.to_le_bytes());                      // bV5Reserved
    buf.extend_from_slice(bgra);
    buf
}

#[cfg(windows)]
unsafe fn global_alloc_and_copy(
    data: &[u8],
) -> Result<windows::Win32::Foundation::HGLOBAL> {
    use anyhow::anyhow;
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    let h = GlobalAlloc(GMEM_MOVEABLE, data.len()).context("GlobalAlloc")?;
    let dst = GlobalLock(h);
    if dst.is_null() {
        return Err(anyhow!("GlobalLock returned null"));
    }
    std::ptr::copy_nonoverlapping(data.as_ptr(), dst as *mut u8, data.len());
    let _ = GlobalUnlock(h);
    Ok(h)
}

#[cfg(not(windows))]
pub fn write_image_and_text(_img: ImageData, text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("init clipboard")?;
    clipboard.set_text(text).context("set text on clipboard")?;
    Ok(())
}
