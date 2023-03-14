use std::env;

use ethers::{providers::{Provider, Http, ProviderError, Middleware}, types::{U256, BlockNumber}, utils::{self, format_units}};


#[tokio::main]
async fn main() {    
    let args: Vec<String> = env::args().collect();
    let provider: Provider<Http> = Provider::<Http>::try_from(
        &args[1]
    ).unwrap();

    // Latest suggestion
    let (max_fee_per_gas, max_priority_fee_per_gas) = estimate_eip1559_fees(
        &provider,
        BlockNumber::Latest,
        None
    ).await.unwrap();
    println!("Latest block's suggestion: max_fee_per_gas {:?} max_priority_fee_per_gas {:?}", format_units(max_fee_per_gas, "gwei").unwrap(), format_units(max_priority_fee_per_gas, "gwei").unwrap());

    // At a block where we saw issues
    let (max_fee_per_gas, max_priority_fee_per_gas) = estimate_eip1559_fees(
        &provider,
        BlockNumber::Number(40340495.into()),
        None
    ).await.unwrap();
    println!("Problematic block 40340495's suggestion: max_fee_per_gas {:?} max_priority_fee_per_gas {:?}", format_units(max_fee_per_gas, "gwei").unwrap(), format_units(max_priority_fee_per_gas, "gwei").unwrap());
}

// copied / adapted from https://github.com/gakonst/ethers-rs/blob/master/ethers-providers/src/rpc/provider.rs#L417-L444
async fn estimate_eip1559_fees(
    provider: &Provider<Http>,
    block: BlockNumber,
    estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
) -> Result<(U256, U256), ProviderError> {
    let base_fee_per_gas = provider
        .get_block(block)
        .await?
        .ok_or_else(|| ProviderError::CustomError("Latest block not found".into()))?
        .base_fee_per_gas
        .ok_or_else(|| ProviderError::CustomError("EIP-1559 not activated".into()))?;

    let fee_history = provider
        .fee_history(
            utils::EIP1559_FEE_ESTIMATION_PAST_BLOCKS,
            block,
            &[utils::EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE],
        )
        .await?;

    // use the provided fee estimator function, or fallback to the default implementation.
    let (max_fee_per_gas, max_priority_fee_per_gas) = if let Some(es) = estimator {
        es(base_fee_per_gas, fee_history.reward)
    } else {
        utils::eip1559_default_estimator(base_fee_per_gas, fee_history.reward)
    };

    Ok((max_fee_per_gas, max_priority_fee_per_gas))
}