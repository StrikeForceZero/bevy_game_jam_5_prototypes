pub trait GetEnabledDisabled {
    fn is_enabled(&self) -> bool;
    fn is_disabled(&self) -> bool {
        !self.is_enabled()
    }
}

pub trait SetEnabledDisabled {
    fn set_enabled_disabled(&mut self, enable: bool);
    fn set_enabled(&mut self) {
        self.set_enabled_disabled(true);
    }
    fn set_disabled(&mut self) {
        self.set_enabled_disabled(false);
    }
}

pub trait EnableDisable: GetEnabledDisabled + SetEnabledDisabled {}

impl GetEnabledDisabled for bool {
    fn is_enabled(&self) -> bool {
        *self
    }
}

impl SetEnabledDisabled for bool {
    fn set_enabled_disabled(&mut self, enable: bool) {
        *self = enable;
    }
}

impl EnableDisable for bool {}
