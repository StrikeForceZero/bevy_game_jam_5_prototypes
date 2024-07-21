use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Field, Index, Token};

pub(crate) fn get_enable_disable_field(
    fields: &Punctuated<Field, Token![,]>,
) -> Result<(Index, Field), syn::Error> {
    let field = if fields.len() == 1 {
        fields.first().map(|f| (0, f))
    } else {
        fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.attrs.iter().any(|a| a.path().is_ident("enable_disable")))
    };

    let Some((field_pos, field)) = field else {
        return Err(syn::Error::new(
            fields.span(),
            "Struct must have exactly 1 field, or a field with #[enable_disable] attribute",
        ));
    };

    Ok((Index::from(field_pos), field.clone()))
}
