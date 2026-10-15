use std::cell::Cell;
use std::collections::HashMap;

/// The values visible to an expression, as a chain of call frames.
///
/// Scoping here is dynamic: a body may read a name bound by any caller, not
/// only its own parameters. Flattening that into one map per call costs a copy
/// of every global on every call, so the frames are linked instead and a call
/// is O(1) to set up.
pub enum Scope<'a> {
    Globals(&'a HashMap<String, f64>),

    Frame {
        params: &'a [String],
        args: &'a [f64],
        parent: &'a Scope<'a>,

        // set when something read a name bound further out than this frame.
        // such a result is not a function of the call's own arguments, so it
        // cannot be memoised against them
        escaped: Cell<bool>,
    },
}

enum Binding {
    /// found in a frame, this many frames out from where the search started
    Frame(usize, f64),
    Global(f64),
}

impl<'a> Scope<'a> {
    pub fn globals(vars: &'a HashMap<String, f64>) -> Self {
        Scope::Globals(vars)
    }

    pub fn frame(params: &'a [String], args: &'a [f64], parent: &'a Scope<'a>) -> Self {
        Scope::Frame {
            params,
            args,
            parent,
            escaped: Cell::new(false),
        }
    }

    /// Whether anything read past this frame while it was current.
    pub fn escaped(&self) -> bool {
        match self {
            Scope::Frame { escaped, .. } => escaped.get(),
            Scope::Globals(_) => false,
        }
    }

    /// The argument in this frame's own slot, without a search.
    pub fn param(&self, index: usize) -> Option<f64> {
        match self {
            Scope::Frame { args, .. } => args.get(index).copied(),
            Scope::Globals(_) => None,
        }
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        match self.find(name)? {
            Binding::Global(value) => Some(value),

            Binding::Frame(distance, value) => {
                // every frame inside the one that bound the name produced a
                // result depending on more than its own arguments. a global is
                // not marked: assigning one already clears the cache
                self.mark_escaped(distance);

                Some(value)
            }
        }
    }

    fn find(&self, name: &str) -> Option<Binding> {
        let mut scope = self;
        let mut distance = 0;

        loop {
            match scope {
                Scope::Globals(vars) => return vars.get(name).copied().map(Binding::Global),

                Scope::Frame {
                    params,
                    args,
                    parent,
                    ..
                } => {
                    if let Some(slot) = params.iter().position(|param| param == name) {
                        return Some(Binding::Frame(distance, args[slot]));
                    }

                    distance += 1;
                    scope = parent;
                }
            }
        }
    }

    fn mark_escaped(&self, distance: usize) {
        let mut scope = self;

        for _ in 0..distance {
            let Scope::Frame {
                escaped, parent, ..
            } = scope
            else {
                return;
            };

            escaped.set(true);
            scope = parent;
        }
    }
}
