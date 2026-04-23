pub trait Magma {
    type Elem: Eq + Copy;
    fn elems(&self) -> impl Iterator<Item=Self::Elem>;
    fn f(&self, _: Self::Elem, _: Self::Elem) -> Self::Elem;

    fn is677(&self) -> bool {
        for x in self.elems() {
            for y in self.elems() {
                if x != self.f(y, self.f(x, self.f(self.f(y, x), y))) {
                    return false;
                }
            }
        }
        true
    }

    fn is255(&self) -> bool {
        for x in self.elems() {
            if x != self.f(self.f(self.f(x, x), x), x) {
                return false;
            }
        }
        true
    }
}
