#![feature(box_patterns)]
#![allow(non_snake_case, unused_imports)]

#[macro_use]
extern crate fstrings;
extern crate proc_macro;
use ::proc_macro::TokenStream;
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use ::quote::{
    quote,
    quote_spanned,
};
use ::syn::{*,
    parse::{
        Parse,
        ParseStream,
    },
    spanned::Spanned,
};
use ::std::{*,
    convert::{TryInto, TryFrom},
    iter::FromIterator,
    result::Result,
};

#[macro_use] mod macros;

#[inline]
fn take<T : Default> (x: &'_ mut T)
  -> T
{
    mem::replace(x, T::default())
}

#[proc_macro_attribute] pub
fn inheritable (params: TokenStream, input: TokenStream)
  -> TokenStream
{
    #[cfg_attr(feature = "verbose-expansions",
        derive(Debug),
    )]
    struct Trait {
        ident: Ident,
        methods: Vec<TraitItemMethod>,
    }

    impl Parse for Trait {
        fn parse (input: ParseStream) -> syn::Result<Self>
        {Ok({
            let item_trait: ItemTrait = input.parse()?;
            let ident: Ident = item_trait.ident;
            let methods: Vec<TraitItemMethod> =
                item_trait
                    .items
                    .into_iter()
                    //  error on non-function items
                    .map(|trait_item| match trait_item {
                        | TraitItem::Method(method) => Ok(method),
                        | _ => parse_error!(
                            trait_item.span(),
                            "`#[inheritable]` only supports methods"
                        ),
                    })
                    // error on non-method functions
                    .map(|x| x.and_then(|method| {
                        let ref sig = method.sig;
                        let mut span = sig.ident.span();
                        match sig.inputs.iter().next() {
                            // & [mut] self
                            | Some(&FnArg::Receiver(_)) => {},
                            // self: & [mut] _
                            | Some(&FnArg::Typed(PatType {
                                pat: box Pat::Ident(PatIdent { ref ident, .. }),
                                ty: box Type::Reference(_),
                                ..
                            }))
                                if ident == "self"
                            => {},
                            // otherwise
                            | opt_arg => {
                                if let Some(arg) = opt_arg {
                                    span = arg.span();
                                }
                                parse_error!(span, concat!(
                                    "associated function requires a ",
                                    "`&self` or `&mut self` receiver",
                                ));
                            },
                        }
                        Ok(method)
                    }))
                    .collect::<Result<_, _>>()?
            ;
            Self {
                ident,
                methods,
            }
        })}
    }

    debug!(concat!(
        "-------------------------\n",
        "#[inheritable({params})]\n",
        "{input}\n",
    ), params=params, input=input);
    let params = TokenStream2::from(params);
    if params.clone().into_iter().next().is_some() {
        error!(params.span(), "Unexpected parameter(s)");
    }

    set_output!( render => ret );
    ret.extend(input.clone());

    let Trait {
        ident: Trait,
        mut methods,
    } = parse_macro_input!(input);

    let InheritsTrait = Ident::new(&f!(
        "Inherits{Trait}"
    ), Span::call_site());

    render! {
        trait #InheritsTrait {
            type Parent
                : ?Sized + #Trait
            ;
            fn parent (self: &'_ Self)
              -> &'_ Self::Parent
            ;
            fn parent_mut (self: &'_ mut Self)
              -> &'_ mut Self::Parent
            ;
        }
    };

    methods
        .iter_mut()
        .for_each(|method| {
            let &mut TraitItemMethod {
                sig: Signature {
                    ref ident,
                    ref generics,
                    ref mut inputs,
                    ..
                },
                ref mut default,
                ref mut semi_token,
                ref mut attrs,
            } = method;
            *attrs = vec![];
            *semi_token = None;
            let mut args: Vec<Ident> =
                Vec::with_capacity(
                    inputs
                        .len()
                        .saturating_sub(1)
                )
            ;
            let mut inputs_iter = take(inputs).into_iter();
            let mut parent_mb_mut = TokenStream2::default();
            *inputs =
                inputs_iter
                    .next()
                    .map(|first_arg| match first_arg {
                        | FnArg::Receiver(Receiver {
                            ref mutability,
                            ..
                        })
                        | FnArg::Typed(PatType {
                            ty: box Type::Reference(TypeReference {
                                ref mutability,
                                ..
                            }),
                            ..
                        })
                        => {
                            parent_mb_mut = if mutability.is_some() {
                                quote! {
                                    parent_mut
                                }
                            } else {
                                quote! {
                                    parent
                                }
                            };
                            first_arg
                        },

                        | _ => unreachable!(),
                    })
                    .into_iter()
                    .chain(
                        inputs_iter
                            .zip(1 ..)
                            .map(|(mut arg, i)| match arg {
                                | FnArg::Typed(PatType { ref mut pat, .. }) => {
                                    let ident = Ident::new(&f!(
                                        "arg_{i}"
                                    ), Span::call_site());
                                    *pat = parse_quote! {
                                        #ident
                                    };
                                    args.push(ident);
                                    arg
                                },

                                | _ => unreachable!("Invalid method signature"),
                            })
                    )
                    .collect()
            ;
            let generics = generics.split_for_impl().1;
            let generics = generics.as_turbofish();
            *default = Some(parse_quote! {
                {
                    /* 100% guaranteed unambiguous version */
                    // <
                    //     <Self as #InheritsTrait>::Parent
                    //     as #Trait
                    // >::#ident #generics (
                    //     #InheritsTrait :: #parent_mb_mut(self),
                    //     #(#args),*
                    // )
                    /* This should nevertheless also be unambiguous */
                    self.#parent_mb_mut()
                        .#ident #generics (
                            #(#args),*
                        )
                }
            });
        })
    ;
    render! {
        impl<__inheritable_T__ : ?Sized + #InheritsTrait> #Trait
            for __inheritable_T__
        {
            #(
                #[inline]
                default
                #methods
            )*
        }
    }

    debug!("=> becomes =>\n\n{}\n-------------------------\n", ret);
    ret
}

#[proc_macro_derive(Inheritance, attributes(inherits))] pub
fn derive_Inheritance (input: TokenStream)
  -> TokenStream
{
    debug!(concat!(
        "-------------------------\n",
        "#[derive(Inheritance)]\n",
        "{input}\n",
        "\n",
    ), input=input);
    set_output!( render => ret );
    let DeriveInput {
        ident: Struct,
        generics,
        data,
        ..
    } = parse_macro_input!(input);
    let fields = match data {
        | Data::Struct(DataStruct { fields, .. }) => fields,
        | Data::Enum(r#enum) => {
            error!(r#enum.enum_token.span(),
                "enums are not supported yet"
            );
        },
        | Data::Union(r#union) => {
            error!(r#union.union_token.span(),
                "unions are not supported yet"
            );
        },
    };
    let (mut iter1, mut iter2);
    let fields: &mut dyn Iterator<Item = Field> = match fields {
        | Fields::Unit => {
            iter1 = iter::empty();
            &mut iter1
        },
        | Fields::Unnamed(fields) => {
            iter2 = fields.unnamed.into_iter();
            &mut iter2
        },
        | Fields::Named(fields) => {
            iter2 = fields.named.into_iter();
            &mut iter2
        },
    };
    let ref inherits: Ident = parse_quote! {
        inherits
    };
    for (i, mut field) in fields.enumerate() {
        let (path_to_InheritsTrait, span) =
            match take(&mut field.attrs)
                    .into_iter()
                    .find_map(|attr| if attr.path.is_ident(inherits) { Some({
                        let span = attr.span();
                        attr.parse_args_with(Path::parse_mod_style)
                            .map(|mut path| {
                                let last =
                                    path.segments
                                        .iter_mut()
                                        .last()
                                        .expect("path has at least one segment")
                                ;
                                let ref Trait = last.ident;
                                let InheritsTrait = Ident::new(&f!(
                                    "Inherits{Trait}"
                                ), span);
                                *last = parse_quote! {
                                    #InheritsTrait
                                };
                                (path, span)
                            })
                    })} else {
                        None
                    })
            {
                | None => continue,
                | Some(Err(err)) => return err.to_compile_error().into(),
                | Some(Ok(inner)) => inner,
            }
        ;
        let field_name =
            if let Some(ref ident) = field.ident {
                quote! {
                    #ident
                }
            } else {
                let i: Index = i.into();
                quote! {
                    #i
                }
            }
        ;
        let ref FieldType = field.ty;
        let (impl_generics, ty_generics, where_clause) =
            generics.split_for_impl()
        ;
        render! { span =>
            impl #impl_generics #path_to_InheritsTrait
                for #Struct #ty_generics
            #where_clause
            {
                type Parent = #FieldType;

                #[inline]
                fn parent (self: &'_ Self)
                  -> &'_ Self::Parent
                {
                    &self.#field_name
                }

                #[inline]
                fn parent_mut (self: &'_ mut Self)
                  -> &'_ mut Self::Parent
                {
                    &mut self.#field_name
                }
            }
        }
    }
    debug!("=> becomes =>\n\n{}\n-------------------------\n", ret);
    ret
}
