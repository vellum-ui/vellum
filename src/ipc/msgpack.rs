use std::io::{self, Read, Write};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::UiEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RustToJsMessage {
    UiEvent { event: UiEvent },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JsToRustMessage {
    SetTitle {
        title: String,
    },
    CreateWidget {
        id: String,
        kind: String,
        parent_id: Option<String>,
        text: Option<String>,
        style_json: Option<String>,
        widget_params_json: Option<String>,
        #[serde(default, with = "serde_bytes")]
        data: Option<Vec<u8>>,
    },
    RemoveWidget {
        id: String,
    },
    SetWidgetText {
        id: String,
        text: String,
    },
    SetWidgetVisible {
        id: String,
        visible: bool,
    },
    SetWidgetStyle {
        id: String,
        style_json: String,
    },
    SetStyleProperty {
        id: String,
        property: String,
        value: String,
    },
    SetWidgetValue {
        id: String,
        value: f64,
    },
    SetWidgetChecked {
        id: String,
        checked: bool,
    },
    ResizeWindow {
        width: u32,
        height: u32,
    },
    CloseWindow,
    ExitApp,
    SetImageData {
        id: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}

pub fn write_msgpack_frame<W, T>(writer: &mut W, value: &T) -> io::Result<()>
where
    W: Write,
    T: Serialize,
{
    let payload = rmp_serde::to_vec_named(value).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("msgpack encode failed: {e}"),
        )
    })?;

    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

pub fn read_msgpack_frame<R, T>(reader: &mut R) -> io::Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut len_bytes = [0_u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes) as usize;

    let mut payload = vec![0_u8; len];
    reader.read_exact(&mut payload)?;

    rmp_serde::from_slice::<T>(&payload).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("msgpack decode failed: {e}"),
        )
    })
}
