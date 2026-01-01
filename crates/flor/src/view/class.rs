pub trait ClassLoader {
    fn load_classes(&mut self, class_str: &[&str]);
}