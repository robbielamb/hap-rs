use std::io::{Error, ErrorKind};

use serde::{ser::{Serialize, Serializer, SerializeStruct}, Deserialize};
use serde_json;
use erased_serde;

use hap_type::HapType;
use event::{Event, EmitterPtr};

mod includes;
pub use characteristic::includes::*;

#[derive(Default)]
pub struct Characteristic<T: Default + Serialize> {
    id: u64,
    accessory_id: u64,
    hap_type: HapType,
    format: Format,
    perms: Vec<Perm>,
    description: Option<String>,
    event_notifications: Option<bool>,

    value: T,
    unit: Option<Unit>,

    max_value: Option<T>,
    min_value: Option<T>,
    step_value: Option<T>,
    max_len: Option<u16>,
    max_data_len: Option<u32>,
    valid_values: Option<Vec<T>>,
    valid_values_range: Option<[T; 2]>,

    readable: Option<Box<Readable<T>>>,
    updatable: Option<Box<Updatable<T>>>,

    event_emitter: Option<EmitterPtr>,
}

impl<T: Default + Serialize> Characteristic<T> where for<'de> T: Deserialize<'de> {
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn set_accessory_id(&mut self, accessory_id: u64) {
        self.accessory_id = accessory_id;
    }

    pub fn get_type(&self) -> &HapType {
        &self.hap_type
    }

    pub fn get_format(&self) -> &Format {
        &self.format
    }

    pub fn get_perms(&self) -> &Vec<Perm> {
        &self.perms
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    pub fn get_event_notifications(&self) -> Option<bool> {
        self.event_notifications
    }

    pub fn set_event_notifications(&mut self, event_notifications: Option<bool>) {
        self.event_notifications = event_notifications;
    }

    pub fn get_value(&mut self) -> Result<&T, Error> {
        let mut val = None;
        if let Some(ref mut readable) = self.readable {
            val = Some(readable.on_read(self.hap_type));
        }
        if let Some(v) = val {
            self.set_value(v)?;
        }

        Ok(&self.value)
    }

    pub fn set_value(&mut self, val: T) -> Result<(), Error> {
        /*if let Some(ref max) = self.max_value {
            if &val > max {
                return Err(Error::new(ErrorKind::Other, "value above max_value"));
            }
        }
        if let Some(ref min) = self.min_value {
            if &val < min {
                return Err(Error::new(ErrorKind::Other, "value below min_value"));
            }
        }*/

        if let Some(ref mut updatable) = self.updatable {
            updatable.on_update(self.hap_type, &self.value, &val);
        }

        if self.event_notifications == Some(true) {
            if let Some(ref event_emitter) = self.event_emitter {
                event_emitter.borrow().emit(Event::CharacteristicValueChanged {
                    aid: self.accessory_id,
                    iid: self.id,
                    value: json!(&val),
                });
            }
        }

        self.value = val;

        Ok(())
    }

    pub fn get_unit(&self) -> &Option<Unit> {
        &self.unit
    }

    pub fn get_max_value(&self) -> &Option<T> {
        &self.max_value
    }

    pub fn set_max_value(&mut self, val: Option<T>) {
        self.max_value = val;
    }

    pub fn get_min_value(&self) -> &Option<T> {
        &self.min_value
    }

    pub fn set_min_value(&mut self, val: Option<T>) {
        self.min_value = val;
    }

    pub fn get_step_value(&self) -> &Option<T> {
        &self.step_value
    }

    pub fn set_step_value(&mut self, val: Option<T>) {
        self.step_value = val;
    }

    pub fn get_max_len(&self) -> Option<u16> {
        self.max_len
    }

    pub fn set_readable(&mut self, readable: impl Readable<T> + 'static) {
        self.readable = Some(Box::new(readable));
    }

    pub fn set_updatable(&mut self, updatable: impl Updatable<T> + 'static) {
        self.updatable = Some(Box::new(updatable));
    }

    pub fn set_event_emitter(&mut self, event_emitter: Option<EmitterPtr>) {
        self.event_emitter = event_emitter;
    }
}

impl<T: Default + Serialize> Serialize for Characteristic<T> where for<'de> T: Deserialize<'de> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Characteristic", 15)?;
        state.serialize_field("iid", &self.id)?;
        state.serialize_field("type", &self.hap_type)?;
        state.serialize_field("format", &self.format)?;
        state.serialize_field("perms", &self.perms)?;
        if let Some(ref description) = self.description {
            state.serialize_field("description", description)?;
        }
        if let Some(ref event_notifications) = self.event_notifications {
            state.serialize_field("ev", event_notifications)?;
        }

        if self.perms.contains(&Perm::PairedRead) {
            state.serialize_field("value", &self.value)?;
        }
        if let Some(ref unit) = self.unit {
            state.serialize_field("unit", unit)?;
        }
        if let Some(ref max_value) = self.max_value {
            state.serialize_field("maxValue", max_value)?;
        }
        if let Some(ref min_value) = self.min_value {
            state.serialize_field("minValue", min_value)?;
        }
        if let Some(ref step_value) = self.step_value {
            state.serialize_field("minStep", step_value)?;
        }
        if let Some(ref max_len) = self.max_len {
            state.serialize_field("maxLen", max_len)?;
        }
        if let Some(ref max_data_len) = self.max_data_len {
            state.serialize_field("maxDataLen", max_data_len)?;
        }
        if let Some(ref valid_values) = self.valid_values {
            state.serialize_field("valid-values", valid_values)?;
        }
        if let Some(ref valid_values_range) = self.valid_values_range {
            state.serialize_field("valid-values-range", valid_values_range)?;
        }
        state.end()
    }
}

pub trait HapCharacteristic: erased_serde::Serialize {
    fn get_id(&self) -> u64;
    fn set_id(&mut self, id: u64);
    fn set_accessory_id(&mut self, accessory_id: u64);
    fn get_type(&self) -> &HapType;
    fn get_format(&self) -> &Format;
    fn get_perms(&self) -> &Vec<Perm>;
    fn get_event_notifications(&self) -> Option<bool>;
    fn set_event_notifications(&mut self, event_notifications: Option<bool>);
    fn get_value(&mut self) -> Result<serde_json::Value, Error>;
    fn set_value(&mut self, value: serde_json::Value) -> Result<(), Error>;
    fn get_unit(&self) -> &Option<Unit>;
    fn get_max_value(&self) -> Option<serde_json::Value>;
    fn get_min_value(&self) -> Option<serde_json::Value>;
    fn get_step_value(&self) -> Option<serde_json::Value>;
    fn get_max_len(&self) -> Option<u16>;
    fn set_event_emitter(&mut self, event_emitter: Option<EmitterPtr>);
}

serialize_trait_object!(HapCharacteristic);

impl<T: Default + Serialize> HapCharacteristic for Characteristic<T> where for<'de> T: Deserialize<'de> {
    fn get_id(&self) -> u64 {
        self.get_id()
    }

    fn set_id(&mut self, id: u64) {
        self.set_id(id)
    }

    fn set_accessory_id(&mut self, accessory_id: u64) {
        self.set_accessory_id(accessory_id)
    }

    fn get_type(&self) -> &HapType {
        self.get_type()
    }

    fn get_format(&self) -> &Format {
        self.get_format()
    }

    fn get_perms(&self) -> &Vec<Perm> {
        self.get_perms()
    }

    fn get_event_notifications(&self) -> Option<bool> {
        self.get_event_notifications()
    }

    fn set_event_notifications(&mut self, event_notifications: Option<bool>) {
        self.set_event_notifications(event_notifications);
    }

    fn get_value(&mut self) -> Result<serde_json::Value, Error> {
        Ok(json!(self.get_value()?))
    }

    fn set_value(&mut self, value: serde_json::Value) -> Result<(), Error> {
        let v;
        // for some reason the controller is setting boolean values
        // either as a boolean or as an integer
        if self.format == Format::Bool && value.is_number() {
            let num_v: u8 = serde_json::from_value(value)?;
            if num_v == 0 {
                v = serde_json::from_value(json!(false))?;
            } else if num_v == 1 {
                v = serde_json::from_value(json!(true))?;
            } else {
                return Err(Error::new(ErrorKind::Other, "invalid value"));
            }
        } else {
            v = serde_json::from_value(value)?;
        }
        self.set_value(v)
    }

    fn get_unit(&self) -> &Option<Unit> {
        self.get_unit()
    }

    fn get_max_value(&self) -> Option<serde_json::Value> {
        match self.get_max_value() {
            &Some(ref v) => Some(json!(v)),
            &None => None,
        }
    }

    fn get_min_value(&self) -> Option<serde_json::Value> {
        match self.get_min_value() {
            &Some(ref v) => Some(json!(v)),
            &None => None,
        }
    }

    fn get_step_value(&self) -> Option<serde_json::Value> {
        match self.get_step_value() {
            &Some(ref v) => Some(json!(v)),
            &None => None,
        }
    }

    fn get_max_len(&self) -> Option<u16> {
        self.get_max_len()
    }

    fn set_event_emitter(&mut self, event_emitter: Option<EmitterPtr>) {
        self.set_event_emitter(event_emitter);
    }
}

pub trait Readable<T: Default + Serialize> {
    fn on_read(&mut self, hap_type: HapType) -> T;
}

pub trait Updatable<T: Default + Serialize> {
    fn on_update(&mut self, hap_type: HapType, old_val: &T, new_val: &T);
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Perm {
    #[serde(rename = "pr")]
    PairedRead,
    #[serde(rename = "pw")]
    PairedWrite,
    #[serde(rename = "ev")]
    Events,
    #[serde(rename = "aa")]
    AdditionalAuthorization,
    #[serde(rename = "tw")]
    TimedWrite,
    #[serde(rename = "hd")]
    Hidden,
}

#[derive(Debug, Clone, Serialize)]
pub enum Unit {
    #[serde(rename = "percentage")]
    Percentage,
    #[serde(rename = "arcdegrees")]
    ArcDegrees,
    #[serde(rename = "celsius")]
    Celsius,
    #[serde(rename = "lux")]
    Lux,
    #[serde(rename = "seconds")]
    Seconds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Format {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "uint8")]
    UInt8,
    #[serde(rename = "uint16")]
    UInt16,
    #[serde(rename = "uint32")]
    UInt32,
    #[serde(rename = "uint64")]
    UInt64,
    #[serde(rename = "int32")]
    Int32,
    #[serde(rename = "tlv8")]
    Tlv8,
    #[serde(rename = "data")]
    Data,
}

impl Default for Format {
    fn default() -> Format {
        Format::String
    }
}
