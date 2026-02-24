use crate::core::{Extension, PhysicalFeature, Vendor};

pub enum Feature {
    Physical(PhysicalFeature),
    Vendor(Vendor),
    Extension(Extension)
}

impl From<PhysicalFeature> for Feature {
    fn from(value: PhysicalFeature) -> Self {
        Feature::Physical(value)
    }
}

impl From<Vendor> for Feature {
    fn from(value: Vendor) -> Self {
        Feature::Vendor(value)
    }
}

impl From<&Vendor> for Feature {
    fn from(value: &Vendor) -> Self {
        Feature::Vendor(*value)
    }
}

impl From<Extension> for Feature {
    fn from(value: Extension) -> Self {
        Feature::Extension(value)
    }
}

impl From<&Extension> for Feature {
    fn from(value: &Extension) -> Self {
        Feature::Extension(*value)
    }
}