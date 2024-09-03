use {
  proc_macro2::{Ident, Span},
  quote::quote,
  std::cmp::{max, min},
  syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Error, Expr, LitInt, Token, Type,
  },
};

#[derive(Debug)]
struct Item {
  start: usize,
  end: usize,
}

impl Parse for Item {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let start = input.parse::<LitInt>()?.base10_parse()?;
    Ok(if let Ok(_) = input.parse::<Token![:]>() {
      Self { start, end: input.parse::<LitInt>()?.base10_parse()? }
    } else {
      Self { start, end: start }
    })
  }
}

struct Input {
  src: Expr,
  in_token: Token![in],
  items: Punctuated<Item, Token![|]>,
}

impl Parse for Input {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    Ok(Self {
      src: input.parse()?,
      in_token: input.parse()?,
      items: Punctuated::parse_separated_nonempty(input)?,
    })
  }
}

fn lit(repr: impl ToString) -> LitInt {
  LitInt::new(&repr.to_string(), Span::call_site())
}

#[proc_macro]
pub fn imm(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let Input { src, items, .. } = parse_macro_input!(input as Input);
  let items = items.into_iter().rev().collect::<Vec<_>>();

  // Useless validation:
  // let mut bitmap = vec![Some(()); bits];
  // for item in items {
  //   match item {
  //     Item::Bit(bit) => bitmap[bit].take().unwrap(),
  //     Item::Range(Range { start, end, .. }) => {
  //       let (start, end) = (min(start, end), max(start, end));
  //       for i in start..end {
  //         bitmap[i].take().unwrap()
  //       }
  //     }
  //   }
  // }

  let mut size = 0;
  let mut lines = Vec::new();

  for Item { start, end } in items {
    let (start, end) = (min(start, end), max(start, end));
    let len = end - start + 1;
    let (shf, start, lit) =
      (lit(size), lit(start), lit(format!("0b{}", "1".repeat(len))));
    lines.push(quote! { ((src >> #shf) & #lit) << #start});
    size += len;
  }
  quote!({ let src = #src; #((#lines)|)* 0 }).into()
}

#[proc_macro]
pub fn slice(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let Input { src, items, .. } = parse_macro_input!(input as Input);
  let items = items.into_iter().rev().collect::<Vec<_>>();

  let mut size = 0;
  let mut lines = Vec::new();

  for Item { start, end } in items {
    let (start, end) = (min(start, end), max(start, end));
    let len = end - start + 1;
    let (start, sz, lit) =
      (lit(start), lit(size), lit(format!("0b{}", "1".repeat(len))));
    lines.push(quote! { ((src >> #start) & #lit) << #sz});
    size += len;
  }
  quote!({ let src = #src; #((#lines)|)* 0 }).into()
}
