//! This module defines the object internal methods.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots

use crate::builtins::{
    object::Object,
    property::{Attribute, Property, PropertyKey},
    value::{same_value, RcString, Value},
};
use crate::BoaProfiler;

impl Object {
    /// Check if the ordinary object is extensible.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-isextensible
    #[inline]
    fn ordinary_is_extensible(&self) -> bool {
        self.extensible
    }

    /// Determine whether it is permitted to add additional properties to this object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#table-5
    #[inline]
    pub fn is_extensible(&self) -> bool {
        self.ordinary_is_extensible()
    }

    /// Prevent further extensions to ordinary object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinaryisextensible
    #[inline]
    fn ordinary_prevent_extensions(&mut self) -> bool {
        self.extensible = false;
        true
    }

    /// Control whether new properties may be added to this object.
    /// Returns `true` if the operation was successful or `false`
    /// if the operation was unsuccessful.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#table-5
    #[inline]
    pub fn prevent_extensions(&mut self) -> bool {
        self.ordinary_prevent_extensions()
    }

    /// Return a bool value indicating whether this ordinay object already has either an own
    /// or inherited property with the specified key.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#table-5
    fn ordinary_has_property(&self, key: &PropertyKey) -> bool {
        if self.get_own_property(key).is_some() {
            return true;
        }

        if let Value::Object(ref object) = self.get_prototype_of() {
            let object = object.borrow();
            object.has_property(key)
        } else {
            false
        }
    }

    /// Return a bool value indicating whether this ordinay object already has either an own
    /// or inherited property with the specified key.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-hasproperty-p
    pub fn has_property(&self, key: &PropertyKey) -> bool {
        self.ordinary_has_property(key)
    }

    /// Delete property.
    pub fn delete(&mut self, key: &PropertyKey) -> bool {
        let desc = if let Some(desc) = self.get_own_property(key) {
            desc
        } else {
            return true;
        };

        if desc.configurable_or(false) {
            self.remove_property(&key.to_string());
            return true;
        }

        false
    }

    /// [[Get]]
    /// https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-get-p-receiver
    pub fn get(&self, key: &PropertyKey) -> Value {
        let desc = self.get_own_property(key);
        if desc.is_none() {
            return if let Some(object) = self.get_prototype_of().as_object() {
                object.get(key)
            } else {
                Value::undefined()
            };
        }

        let desc = desc.unwrap();
        if desc.is_data_descriptor() {
            return desc.value.clone().expect("failed to extract value");
        }

        let getter = desc.get.clone();
        if getter.is_none() || getter.expect("Failed to get object").is_undefined() {
            return Value::undefined();
        }

        // TODO: Call getter from here!
        Value::undefined()
    }

    /// [[Set]]
    /// <https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-set-p-v-receiver>
    pub fn set(&mut self, key: &PropertyKey, val: Value) -> bool {
        let _timer = BoaProfiler::global().start_event("Object::set", "object");

        // Fetch property key
        let mut own_desc = if let Some(desc) = self.get_own_property(key) {
            desc
        } else {
            let parent = self.get_prototype_of();
            if !parent.is_null() {
                // TODO: come back to this
            }
            Property::data_descriptor(
                Value::undefined(),
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
        };
        // [3]
        if own_desc.is_data_descriptor() {
            if !own_desc.writable() {
                return false;
            }

            // Change value on the current descriptor
            own_desc = own_desc.value(val);
            return self.define_own_property(key, own_desc);
        }
        // [4]
        debug_assert!(own_desc.is_accessor_descriptor());
        match own_desc.set {
            None => false,
            Some(_) => {
                unimplemented!();
            }
        }
    }

    /// Define an own property.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-defineownproperty-p-desc
    pub fn define_own_property(&mut self, property_key: &PropertyKey, desc: Property) -> bool {
        let _timer = BoaProfiler::global().start_event("Object::define_own_property", "object");

        let mut current = self
            .get_own_property(property_key)
            .unwrap_or_else(Property::empty);
        let extensible = self.is_extensible();

        // https://tc39.es/ecma262/#sec-validateandapplypropertydescriptor
        // There currently isn't a property, lets create a new one
        if current.value.is_none() || current.value.as_ref().expect("failed").is_undefined() {
            if !extensible {
                return false;
            }

            self.insert_property(property_key, desc);
            return true;
        }
        // If every field is absent we don't need to set anything
        if desc.is_none() {
            return true;
        }

        // 4
        if !current.configurable_or(false) {
            if desc.configurable_or(false) {
                return false;
            }

            if desc.enumerable_or(false) != current.enumerable_or(false) {
                return false;
            }
        }

        // 5
        if desc.is_generic_descriptor() {
            // 6
        } else if current.is_data_descriptor() != desc.is_data_descriptor() {
            // a
            if !current.configurable() {
                return false;
            }
            // b
            if current.is_data_descriptor() {
                // Convert to accessor
                current.value = None;
                current.attribute.remove(Attribute::WRITABLE);
            } else {
                // c
                // convert to data
                current.get = None;
                current.set = None;
            }

            self.insert_property(property_key, current);
        // 7
        } else if current.is_data_descriptor() && desc.is_data_descriptor() {
            // a
            if !current.configurable() && !current.writable() {
                if desc.writable_or(false) {
                    return false;
                }

                if desc.value.is_some()
                    && !same_value(
                        &desc.value.clone().unwrap(),
                        &current.value.clone().unwrap(),
                    )
                {
                    return false;
                }

                return true;
            }
        // 8
        } else {
            if !current.configurable() {
                if desc.set.is_some()
                    && !same_value(&desc.set.clone().unwrap(), &current.set.clone().unwrap())
                {
                    return false;
                }

                if desc.get.is_some()
                    && !same_value(&desc.get.clone().unwrap(), &current.get.clone().unwrap())
                {
                    return false;
                }
            }

            return true;
        }
        // 9
        self.insert_property(property_key, desc);
        true
    }

    /// The specification returns a Property Descriptor or Undefined.
    ///
    /// These are 2 separate types and we can't do that here.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getownproperty-p
    pub fn get_own_property(&self, property_key: &PropertyKey) -> Option<Property> {
        let _timer = BoaProfiler::global().start_event("Object::get_own_property", "object");

        // Prop could either be a String or Symbol
        match property_key {
            PropertyKey::String(ref st) => self.properties().get(st).map(|v| {
                let mut d = Property::empty();
                if v.is_data_descriptor() {
                    d.value = v.value.clone();
                } else {
                    debug_assert!(v.is_accessor_descriptor());
                    d.get = v.get.clone();
                    d.set = v.set.clone();
                }
                d.attribute = v.attribute;
                d
            }),
            PropertyKey::Symbol(ref symbol) => {
                self.symbol_properties().get(&symbol.hash()).map(|v| {
                    let mut d = Property::empty();
                    if v.is_data_descriptor() {
                        d.value = v.value.clone();
                    } else {
                        debug_assert!(v.is_accessor_descriptor());
                        d.get = v.get.clone();
                        d.set = v.set.clone();
                    }
                    d.attribute = v.attribute;
                    d
                })
            }
        }
    }

    /// `Object.setPropertyOf(obj, prototype)`
    ///
    /// This method sets the prototype (i.e., the internal `[[Prototype]]` property)
    /// of a specified object to another object or `null`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-setprototypeof-v
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/setPrototypeOf
    pub fn set_prototype_of(&mut self, val: Value) -> bool {
        debug_assert!(val.is_object() || val.is_null());
        let current = self.prototype.clone();
        if same_value(&current, &val) {
            return true;
        }
        if !self.is_extensible() {
            return false;
        }
        let mut p = val.clone();
        let mut done = false;
        while !done {
            if p.is_null() {
                done = true
            } else if same_value(&Value::from(self.clone()), &p) {
                return false;
            } else {
                let prototype = p
                    .as_object()
                    .expect("prototype should be null or object")
                    .prototype
                    .clone();
                p = prototype;
            }
        }
        self.prototype = val;
        true
    }

    /// Returns either the prototype or null
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots-getprototypeof
    #[inline]
    pub fn get_prototype_of(&self) -> Value {
        self.prototype.clone()
    }

    /// Helper function for property insertion.
    #[inline]
    pub(crate) fn insert_property<N>(&mut self, name: N, p: Property)
    where
        N: Into<RcString>,
    {
        self.properties.insert(name.into(), p);
    }

    /// Helper function for property removal.
    #[inline]
    pub(crate) fn remove_property(&mut self, name: &str) {
        self.properties.remove(name);
    }

    /// Inserts a field in the object `properties` without checking if it's writable.
    ///
    /// If a field was already in the object with the same name that a `Some` is returned
    /// with that field, otherwise None is retuned.
    #[inline]
    pub(crate) fn insert_field<N>(&mut self, name: N, value: Value) -> Option<Property>
    where
        N: Into<RcString>,
    {
        self.properties.insert(
            name.into(),
            Property::data_descriptor(
                value,
                Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            ),
        )
    }

    /// This function returns an Optional reference value to the objects field.
    ///
    /// if it exist `Some` is returned with a reference to that fields value.
    /// Otherwise `None` is retuned.
    #[inline]
    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.properties.get(name).and_then(|x| x.value.as_ref())
    }
}