pub struct Views<V> {
    views: Vec<V>,
}

impl<V> Drop for Views<V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<V> Views<V> {
    pub fn new(initial: V) -> Self {
        let mut new_instance = Self {
            views: Vec::with_capacity(3),
        };

        new_instance.push(initial);
        new_instance
    }

    pub fn push(&mut self, view: V) {
        self.views.push(view);
    }

    pub fn pop(&mut self) {
        self.views.pop();
    }

    pub fn change(&mut self, new_view: V) {
        let count = self.views.len();

        if count > 0 {
            self.views[count - 1] = new_view;
        }
    }

    pub fn clear(&mut self) {
        while !self.is_empty() {
            self.pop();
        }
    }

    pub fn current(&self) -> Option<&V> {
        self.views.last()
    }

    pub fn current_mut(&mut self) -> Option<&mut V> {
        self.views.last_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Views;

    #[derive(Eq, PartialEq, Debug)]
    enum TestViews {
        View1,
        View2,
    }

    #[test]
    fn test_new() {
        let views = Views::new(TestViews::View1);

        assert!(!views.is_empty());
    }

    #[test]
    fn test_push() {
        let mut views = Views::new(TestViews::View1);

        views.push(TestViews::View2);

        assert_eq!(Some(&TestViews::View2), views.current());
    }

    #[test]
    fn test_pop() {
        let mut views = Views::new(TestViews::View1);

        views.pop();

        assert!(views.is_empty());
    }
}
