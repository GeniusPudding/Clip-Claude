use anyhow::{Context, Result};
use arboard::ImageData;

// --- Windows -------------------------------------------------------------

#[cfg(windows)]
const CF_DIB: u32 = 8;
#[cfg(windows)]
const CF_UNICODETEXT: u32 = 13;
#[cfg(windows)]
const BI_BITFIELDS: u32 = 3;

#[cfg(windows)]
pub fn get_sequence_number() -> u32 {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    unsafe { GetClipboardSequenceNumber() }
}

#[cfg(windows)]
pub fn write_image_and_text(img: &ImageData, text: &str) -> Result<()> {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };

    let width = img.width as i32;
    let height = img.height as i32;

    let dib = build_dib_compatible(width, height, &img.bytes);

    let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let mut utf16_bytes: Vec<u8> = Vec::with_capacity(utf16.len() * 2);
    for c in &utf16 {
        utf16_bytes.extend_from_slice(&c.to_le_bytes());
    }

    unsafe {
        let h_dib = global_alloc_and_copy(&dib)?;
        let h_text = global_alloc_and_copy(&utf16_bytes)?;

        OpenClipboard(None).context("OpenClipboard")?;
        EmptyClipboard().context("EmptyClipboard")?;
        SetClipboardData(CF_DIB, Some(HANDLE(h_dib.0)))
            .context("SetClipboardData(CF_DIB)")?;
        SetClipboardData(CF_UNICODETEXT, Some(HANDLE(h_text.0)))
            .context("SetClipboardData(CF_UNICODETEXT)")?;
        CloseClipboard().context("CloseClipboard")?;
    }
    Ok(())
}

#[cfg(windows)]
pub fn write_image_only(img: &ImageData) -> Result<()> {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
    };

    let width = img.width as i32;
    let height = img.height as i32;

    let dib = build_dib_compatible(width, height, &img.bytes);

    unsafe {
        let h_dib = global_alloc_and_copy(&dib)?;

        OpenClipboard(None).context("OpenClipboard")?;
        EmptyClipboard().context("EmptyClipboard")?;
        SetClipboardData(CF_DIB, Some(HANDLE(h_dib.0)))
            .context("SetClipboardData(CF_DIB)")?;
        CloseClipboard().context("CloseClipboard")?;
    }
    Ok(())
}

// Layout matches what .NET System.Windows.Forms.Clipboard::SetImage emits:
// 40-byte BITMAPINFOHEADER, positive biHeight (bottom-up), 32-bit BI_BITFIELDS,
// R/G/B masks, BGRA pixels. Top-down or 24-bit BI_RGB get rejected by some
// chat apps (LINE, Telegram, Discord).
#[cfg(windows)]
fn build_dib_compatible(width: i32, height: i32, rgba: &[u8]) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let pixel_size = (w * h * 4) as u32;

    let mut buf = Vec::with_capacity(40 + 12 + pixel_size as usize);
    buf.extend_from_slice(&40u32.to_le_bytes());
    buf.extend_from_slice(&width.to_le_bytes());
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&32u16.to_le_bytes());
    buf.extend_from_slice(&BI_BITFIELDS.to_le_bytes());
    buf.extend_from_slice(&pixel_size.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0x00FF_0000u32.to_le_bytes());
    buf.extend_from_slice(&0x0000_FF00u32.to_le_bytes());
    buf.extend_from_slice(&0x0000_00FFu32.to_le_bytes());

    for y in (0..h).rev() {
        for x in 0..w {
            let i = (y * w + x) * 4;
            buf.push(rgba[i + 2]);
            buf.push(rgba[i + 1]);
            buf.push(rgba[i]);
            buf.push(rgba[i + 3]);
        }
    }
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

// --- macOS ---------------------------------------------------------------

#[cfg(target_os = "macos")]
pub fn get_sequence_number() -> u32 {
    use objc2_app_kit::NSPasteboard;
    unsafe {
        let pb = NSPasteboard::generalPasteboard();
        pb.changeCount() as u32
    }
}

#[cfg(target_os = "macos")]
pub fn write_image_and_text(img: &ImageData, text: &str) -> Result<()> {
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::{NSData, NSString};

    let png_bytes = encode_png(&img.bytes, img.width as u32, img.height as u32)?;

    unsafe {
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();

        let ns_data = NSData::with_bytes(&png_bytes);
        let png_ty = NSString::from_str("public.png");
        let _ = pb.setData_forType(Some(&ns_data), &png_ty);

        let ns_str = NSString::from_str(text);
        let str_ty = NSString::from_str("public.utf8-plain-text");
        let _ = pb.setString_forType(&ns_str, &str_ty);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn write_image_only(img: &ImageData) -> Result<()> {
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::{NSData, NSString};

    let png_bytes = encode_png(&img.bytes, img.width as u32, img.height as u32)?;

    unsafe {
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();

        let ns_data = NSData::with_bytes(&png_bytes);
        let png_ty = NSString::from_str("public.png");
        let _ = pb.setData_forType(Some(&ns_data), &png_ty);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn encode_png(rgba: &[u8], w: u32, h: u32) -> Result<Vec<u8>> {
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder};
    let mut buf = Vec::new();
    PngEncoder::new(&mut buf)
        .write_image(rgba, w, h, ExtendedColorType::Rgba8)
        .context("encode PNG")?;
    Ok(buf)
}

// --- other (Linux, BSDs) -------------------------------------------------

#[cfg(not(any(windows, target_os = "macos")))]
pub fn get_sequence_number() -> u32 {
    0
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn write_image_and_text(_img: &ImageData, text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("init clipboard")?;
    clipboard.set_text(text).context("set text on clipboard")?;
    Ok(())
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn write_image_only(_img: &ImageData) -> Result<()> {
    Ok(())
}
