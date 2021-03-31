use proc_macro::TokenStream;
use syn::{ItemFn, FnArg, MetaList, NestedMeta, Lit, Meta};
//use syn::ToTokens;
use quote::ToTokens;
use anyhow::{anyhow, bail, Context, Result};

const USAGE: &'static str =
"USAGE:\n\
The `query` attribute must specify at least two arguments indicating the month and the year of the \
dataset to be used in the query, eg: \n\
        \t#[query(May, 2020)]\n\
\n\
In addition, the attribute tag can also optionally specify which subsets to use, eg: \n\
        \t#[query(May, 2020, subset(\"small projects\"))]\n\
        \t#[query(May, 2020, subsets(\"C\", \"Python\", \"small projects\"))]\n\
\n\
If none are specified, the entire dataset will be used.\n\
\n\
The `query` attribute can also optionally specify a numerical seed value for use with random \
number generation:\n\
        \t#[query(May, 2020, seed(42)]\n\
The default seed value is 0.\n";

// const SIGNATURE: &'static str =
//     "";

#[derive(Debug, Eq, Hash, PartialEq, Ord, PartialOrd, Clone, Copy)]
enum Month {
    January, February, March, April, May, June, July, August, September, October, November, December
}

impl Month {
    pub fn from<S>(string: S) -> Result<Self> where S: Into<String> {
        match string.into().to_lowercase().as_str() {
            "jan" | "january"   => Ok(Self::January),
            "feb" | "february"  => Ok(Self::February),
            "mar" | "march"     => Ok(Self::March),
            "apr" | "april"     => Ok(Self::April),
            "may"               => Ok(Self::May),
            "jun" | "june"      => Ok(Self::June),
            "jul" | "july"      => Ok(Self::July),
            "aug" | "august"    => Ok(Self::August),
            "sep" | "september" => Ok(Self::September),
            "oct" | "october"   => Ok(Self::October),
            "nov" | "november"  => Ok(Self::November),
            "dec" | "december"  => Ok(Self::December),
            string => Err(anyhow!("Cannot parse string as month name: \"{}\"", string)),
        }
    }
}

type Year = u32;
type Subset = String;
type Seed = u128;

fn parse_attributes(arguments: TokenStream) -> Result<(Month, Year, Vec<Subset>, Seed)> {
    let attributes: MetaList =
        syn::parse_str(&format!("_({})", arguments.to_string()))
            .with_context(|| format!("Could not parse argument list: {}", arguments.to_string()))?;

    if attributes.nested.len() < 2 {
        bail!("Expected at least two arguments, but found {}", attributes.nested.len());
    }

    let mut month: Option<Month> = None;
    let mut year: Option<Year> = None;
    let mut subsets: Vec<Subset> = vec![];
    let mut seed: Seed = 0;

    for attribute in attributes.nested {
        match attribute {
            NestedMeta::Meta(meta) => {
                match meta {
                    Meta::Path(path) => {
                        match path.get_ident().to_token_stream().to_string().as_str() {
                            string if Month::from(string).is_ok() =>
                                month = Some(Month::from(string)?),
                            string => bail!("Unknown attribute {}.", string),
                        }
                    }
                    Meta::List(mut list) => {
                        match list.path.get_ident().to_token_stream().to_string().as_str() {
                            "seed" => {
                                if list.nested.len() != 1 {
                                    bail!("Expected attribute `seed` to have 1 argument, but {} \
                                           were provided.", list.nested.len());
                                }
                                match list.nested.pop().unwrap().value() {
                                    NestedMeta::Lit(Lit::Int(literal)) => {
                                        seed = literal.to_token_stream().to_string()
                                            .parse::<u128>()
                                            .with_context(|| format!("Expected a number \
                                                      representing a seed, but cannot parse \
                                                      literal {} as a 128-byte unsigned integer.",
                                                      literal))?
                                    },
                                    argument =>
                                        bail!("Expected attribute `seed` to have a number literal \
                                               as an argument, but found {:?}.", argument),
                                }
                            },
                            "subset" | "subsets" => {
                                for argument in list.nested {
                                    match argument {
                                        NestedMeta::Meta(Meta::Path(path)) =>
                                            subsets.push(path.to_token_stream().to_string()),
                                        NestedMeta::Lit(Lit::Str(literal)) =>
                                            subsets.push(literal.value()),
                                        _ => bail!("Expected attribute `subsets` to have \
                                                    identifiers or strings as arguments, but \
                                                    found {:?}.", argument)
                                    }
                                }
                            },
                            string => bail!("Unknown attribute {}.", string),
                        }
                    }
                    Meta::NameValue(_name_value) => { unimplemented!() }
                }
            }
            NestedMeta::Lit(literal) =>
                if let Lit::Int(literal) = literal {
                    let n: i32 =
                        literal.to_token_stream().to_string()
                            .parse::<i32>()
                            .with_context(|| format!("Expected a number representing a year, but \
                                                      cannot parse literal {} as integer.",
                                                      literal))?;
                    if n < 0 {
                        bail!("Expected a number representing a year, but {} is a negative \
                               integer.", n)
                    }
                    year = Some(n as u32)
                } else {
                    bail!("Expected a number representing a year, but literal {} cannot be \
                           interpreted as a year.", literal.to_token_stream().to_string())
                }
        }
    }

    let month: Month =
        month.with_context(|| format!("Expected a month to be specified, but none was found."))?;
    let year: u32 =
        year.with_context(|| format!("Expected a year to be specified, but none was found."))?;

    // println!("month:   {:?}", month);
    // println!("year:    {}",   year);
    // println!("subsets: {:?}", subsets);
    // println!("seed:    {}",   seed);

    Ok((month, year, subsets, seed))
}

#[proc_macro_attribute]
pub fn djanco(attributes: TokenStream, item: TokenStream) -> TokenStream {

    let function: ItemFn = syn::parse(item.clone())
        .expect(&format!("Could not parse function: {}", item.to_string()));

    if function.vis.to_token_stream().to_string() != "pub" {
        panic!("A function tagged as `query` must be public, but function `{}` has visibility `{}`",
               function.sig.ident.to_string(),
               function.vis.to_token_stream().to_string());
    }

    let arguments: Vec<FnArg> = function.sig.inputs.clone().into_iter().collect();

    if arguments.len() != 3 {
        panic!("A function tagged as `query` must have 3 arguments, but function `{}` has {}: {:?}",
               function.sig.ident.to_string(),
               function.sig.inputs.len(),
               function.sig.inputs.to_token_stream().to_string());
    }

    // TODO It would be nice to verify the signature here, but
    //      given the amount of information available, it's not
    //      easy to do in the general case, so the options are either
    //      to check with stricter requirements than necessary,
    //      do a warning instead of an error if the check fails, or
    //      forgo checking and let the compiler complain when the runner
    //      bootstrap code is generated.

    // if let Some(FnArg::Typed(argument)) = arguments.get(0) {
    //     println!("db := {:?}", argument);
    //     println!("   ty {:?}", argument.ty);
    //     println!("   attrs {:?}", argument.attrs);
    //     println!("   pat {:?}", argument.pat);
    //     //let pattern: &Option<Ident> = argument.pat;
    //     //let typ: Ty = argument.ty;
    // } else {
    //     panic!("unexpected!");
    // }

    let (_month, _year, _subsets, _seed) =
        parse_attributes(attributes)
            .with_context(|| format!("Problem parsing attribute (cause at the bottom)\n\n{}", USAGE))
            .unwrap();

    item
}