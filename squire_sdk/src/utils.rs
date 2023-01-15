pub struct Url<const N: usize> {
    pub(crate) route: &'static str,
    pub(crate) replacements: [&'static str; N],
}

impl Url<0> {
    pub const fn from(route: &'static str) -> Self {
        Self {
            route,
            replacements: [],
        }
    }
}

impl<const N: usize> Url<N> {
    pub const fn new(route: &'static str, replacements: [&'static str; N]) -> Self {
        Self {
            route,
            replacements,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        self.route
    }


    pub fn replace(&self, values: &[&str; N]) -> String {
        let mut digest = self.route.to_string();
        for (pattern, value) in self.replacements.iter().zip(values.iter()) {
            digest = digest.replacen(pattern, value, 1);
        }
        digest
    }
}

impl Url<1> {
    pub fn insert<S: ToString>(&self, value: S) -> String {
        self.route.replace(self.replacements[0], &value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::Url;

    const SIMPLE_ROUTE: Url<0> = Url::from("/api/v1/test");
    const ANOTHER_SIMPLE_ROUTE: Url<0> = Url::new("/api/v1/another_test", []);
    const SINGLE_REPLACEMENT: Url<1> = Url::new("/api/v1/:id/test", [":id"]);
    const TRIPLE_REPLACEMENT: Url<3> = Url::new("/api/v1/:id/:id/:id/test", [":id"; 3]);
    const TRIPLE_UNIQUE_REPLACEMENT: Url<3> =
        Url::new("/api/v1/:id1/:id2/:id3/test", [":id1", ":id2", ":id3"]);

    #[test]
    fn simple_test() {
        assert_eq!(SIMPLE_ROUTE.as_str(), "/api/v1/test");
        assert_eq!(&SIMPLE_ROUTE.replace(&[]), "/api/v1/test");
        assert_eq!(ANOTHER_SIMPLE_ROUTE.as_str(), "/api/v1/another_test");
        assert_eq!(&ANOTHER_SIMPLE_ROUTE.replace(&[]), "/api/v1/another_test");
    }

    #[test]
    fn single_replacement_test() {
        assert_eq!(SINGLE_REPLACEMENT.as_str(), "/api/v1/:id/test");
        let id = Uuid::new_v4();
        assert_eq!(SINGLE_REPLACEMENT.insert(id), format!("/api/v1/{id}/test"));
        assert_eq!(
            SINGLE_REPLACEMENT.replace(&[&id.to_string()]),
            format!("/api/v1/{id}/test")
        );
    }

    #[test]
    fn triple_replacement_test() {
        assert_eq!(TRIPLE_REPLACEMENT.as_str(), "/api/v1/:id/:id/:id/test");
        let id = Uuid::new_v4();
        assert_eq!(
            TRIPLE_REPLACEMENT.replace(&[&id.to_string(); 3]),
            format!("/api/v1/{id}/{id}/{id}/test")
        );
        let id_one = Uuid::new_v4();
        let id_two = Uuid::new_v4();
        let id_three = Uuid::new_v4();
        assert_eq!(
            TRIPLE_REPLACEMENT.replace(&[
                &id_one.to_string(),
                &id_two.to_string(),
                &id_three.to_string()
            ]),
            format!("/api/v1/{id_one}/{id_two}/{id_three}/test")
        );
        assert_eq!(
            TRIPLE_UNIQUE_REPLACEMENT.as_str(),
            "/api/v1/:id1/:id2/:id3/test"
        );
        let id = Uuid::new_v4();
        assert_eq!(
            TRIPLE_UNIQUE_REPLACEMENT.replace(&[&id.to_string(); 3]),
            format!("/api/v1/{id}/{id}/{id}/test")
        );
        let id_one = Uuid::new_v4();
        let id_two = Uuid::new_v4();
        let id_three = Uuid::new_v4();
        assert_eq!(
            TRIPLE_UNIQUE_REPLACEMENT.replace(&[
                &id_one.to_string(),
                &id_two.to_string(),
                &id_three.to_string()
            ]),
            format!("/api/v1/{id_one}/{id_two}/{id_three}/test")
        );
    }
}
