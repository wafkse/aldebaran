//! Forwarding report derive integration tests.

#![cfg(feature = "codegen")]

#[cfg(test)]
mod tests {
    use aldebaran_baseline::span::Span;
    use aldebaran_report::codegen::Report;
    use aldebaran_report::{
        annotated::{Annotated, Annotations, InlineAnnotations, label::Label},
        prelude::Report,
    };

    /// A failure with an opener location.
    #[derive(Debug, Clone, Report)]
    #[aldebaran(crate)]
    #[report(title = "inner", annotations = labels)]
    struct Inner {
        labels: InlineAnnotations<Label<&'static str>, 1>,
    }

    /// A failure with only its primary location.
    #[derive(Debug, Clone, Report)]
    #[aldebaran(crate)]
    #[report(title = "single", annotations = labels)]
    struct Single {
        labels: InlineAnnotations<Label<&'static str>, 0>,
    }

    /// A concrete sum with differently sized annotation collections.
    #[derive(Debug, Clone, Report)]
    #[aldebaran(crate)]
    #[report(forward)]
    enum Family {
        Inner(Inner),

        Single(Single),
    }

    /// A generic report family preserving its payload annotations.
    #[derive(Debug, Clone, Report)]
    #[aldebaran(crate)]
    #[report(forward)]
    enum Outer<E> {
        Single(Single),

        Nested(E),
    }

    #[test]
    fn forwarding_preserves_annotations_through_nested_generic_enums() {
        let labels = InlineAnnotations::new(Label::new("primary", Span::unit(5)), [Label::new("opener", Span::unit(1))]);
        let error = Outer::Nested(Family::Inner(Inner { labels }));
        let list = error.annotations().list().expect("annotations");

        assert_eq!(list.first().target(), Span::unit(5));
        assert_eq!(list.rest()[0].target(), Span::unit(1));
        assert_eq!(error.title(), "inner");
        assert_eq!(error.target().map(|span| span.range()), Some(1..6));

        let source = "abcdef";
        let attached = error.attach(source);

        assert_eq!(attached.annotations().list().expect("attached labels").rest().len(), 1);

        let single = Single {
            labels: InlineAnnotations::new(Label::new("only", Span::unit(0)), []),
        };
        let family = Family::Single(single.clone());
        let outer = Outer::<Family>::Single(single);

        assert!(family.annotations().list().expect("single").rest().is_empty());
        assert_eq!(outer.title(), "single");
    }
}
