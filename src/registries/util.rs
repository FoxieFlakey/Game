use futures::{StreamExt, stream::FuturesUnordered};

use crate::{registry::Registry, util::identifier::Identifier};

pub async fn build_registry<ArgsIterator, LoaderFuture, Loader, LoaderArgs, LoadedData, LoadError>(
    iterator: ArgsIterator,
    mut loader: Loader,
) -> Result<Registry<LoadedData>, Vec<LoadError>>
where
    Loader: FnMut(LoaderArgs) -> LoaderFuture,
    LoaderFuture: Future<Output = Result<(Identifier, LoadedData), LoadError>>,
    ArgsIterator: Iterator<Item = LoaderArgs>,
{
    let mut registry = Registry::new();

    let mut failures = Vec::new();
    let mut tasks = FuturesUnordered::new();
    const LOAD_CONCURRENY_COUNT: usize = 8;

    for args in iterator {
        if tasks.len() >= LOAD_CONCURRENY_COUNT {
            if let Some(ret) = tasks.next().await {
                match ret {
                    Ok((identifier, data)) => registry.add(identifier, data),
                    Err(e) => failures.push(e),
                }
            }
        }

        tasks.push(loader(args));
    }

    // Finish the rest of futures that already in loading
    while let Some(ret) = tasks.next().await {
        match ret {
            Ok((identifier, data)) => registry.add(identifier, data),
            Err(e) => failures.push(e),
        }
    }

    if failures.len() > 0 {
        Err(failures)
    } else {
        Ok(registry)
    }
}
