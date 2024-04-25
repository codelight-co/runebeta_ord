/*
 * This script can run multiple time to update rune entry data: mints, supply, burned, remaining
*/
-- Update rune entry to initial value
UPDATE transaction_rune_entries e SET 
    mints = 0,
    supply = e.premine,
    burned = 0,
    remaining = e.cap;

-- Aggregate data into rune_stats aggregate table
SELECT 
  rune_id, sum(mints) as mints, sum(mints * mint_amount) as mint_amount , sum(burned) as burned 
INTO rune_stats_agg 
from rune_stats rs 
where aggregated =true group by rune_id ;

-- Update rune entries base on aggregated data
UPDATE transaction_rune_entries e SET 
    mints = e.mints + s.mints, 
    supply = e.supply + s.mint_amount,
    burned = e.burned + s.burned,  
    remaining = e.remaining - s.mints
  FROM rune_stats_agg s 
  WHERE e.rune_id = s.rune_id;

DELETE TABLE rune_stats_agg;