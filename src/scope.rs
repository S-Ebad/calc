use std::collections::HashMap;

/// The values visible to an expression: a call's own parameters, over the
/// globals.
///
/// A body sees its own parameters first and the globals otherwise. It does not
/// see a caller's parameters: which variable a name refers to should not depend
/// on who called, or naming a parameter would silently capture a name the
/// callee meant to read from the globals.
///
/// Globals are borrowed rather than copied, so entering a call costs nothing
/// proportional to how much is in scope.
pub struct Scope<'a> {
    params: &'a [String],
    args: &'a [f64],
    globals: &'a HashMap<String, f64>,
}

impl<'a> Scope<'a> {
    pub fn globals(globals: &'a HashMap<String, f64>) -> Self {
        Self {
            params: &[],
            args: &[],
            globals,
        }
    }

    /// The scope a call's body runs in: its own parameters over the same
    /// globals, with the caller's parameters deliberately left behind.
    pub fn frame(&self, params: &'a [String], args: &'a [f64]) -> Self {
        Self {
            params,
            args,
            globals: self.globals,
        }
    }

    /// The argument in a given slot, without a search.
    pub fn param(&self, slot: usize) -> Option<f64> {
        self.args.get(slot).copied()
    }

    pub fn has_param(&self, name: &str) -> bool {
        self.params.iter().any(|param| param == name)
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        if let Some(slot) = self.params.iter().position(|param| param == name) {
            return Some(self.args[slot]);
        }

        self.globals.get(name).copied()
    }
}
