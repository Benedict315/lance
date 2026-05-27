pub fn cancel_bid(env: Env, job_id: u64, freelancer: Address) -> i128 {
    ensure_initialized(&env);
    freelancer.require_auth();

    let job = read_job(&env, job_id);
    if job.status != JobStatus::Open {
        panic_with_error!(&env, JobRegistryError::JobNotOpen);
    }

    let bidder_key = DataKey::BidIndex(job_id, freelancer.clone());
    let bid_index: u32 = env
        .storage()
        .persistent()
        .get(&bidder_key)
        .unwrap_or_else(|| panic_with_error!(&env, JobRegistryError::BidNotFound));
    let bid = read_bid_at(&env, job_id, bid_index);
    let bid_count = read_bid_count(&env, job_id);
    let last_index = bid_count
        .checked_sub(1)
        .unwrap_or_else(|| panic_with_error!(&env, JobRegistryError::BidIndexOutOfBounds));

    if bid_index != last_index {
        let moved_bid = read_bid_at(&env, job_id, last_index);
        env.storage()
            .persistent()
            .set(&DataKey::Bid(job_id, bid_index), &moved_bid);
        env.storage().persistent().set(
            &DataKey::BidIndex(job_id, moved_bid.freelancer.clone()),
            &bid_index,
        );
    }

    env.storage()
        .persistent()
        .remove(&DataKey::Bid(job_id, last_index));
    env.storage().persistent().remove(&bidder_key);
    env.storage()
        .persistent()
        .set(&DataKey::BidCount(job_id), &last_index);
    credit_refund(&env, freelancer.clone(), bid.collateral_stroops);
    // ... rest of the function