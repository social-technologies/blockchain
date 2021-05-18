use crate::EraIndex;
use sp_runtime::{Perbill, Percent, SaturatedConversion, traits::{AtLeast32BitUnsigned, Saturating, Zero}};

/// The total payout to all validators (and their nominators) per era and maximum payout.
///
/// Defined as such:
/// `maximum-payout = 1.000233278 * total-tokens (1 - 0.00005) ^ era-index`
/// `staker-payout = maximum_payout * 0.99`
pub fn compute_total_payout<N>(
    era_index: EraIndex,
    total_tokens: N,
    total_issuance: N,
) -> (N, N) where N: AtLeast32BitUnsigned + Clone {
    if era_index < 360_000 {
        // If era < 360,000 mint according to inflation formula
        // Hourly Inflation rate is 0.00351%
        let inflation_rate = Perbill::from_rational_approximation(3_151u128, 1_000_000_000u128);
        // Hourly Decay rate is 0.000555%
        let inflation_decay = Perbill::from_rational_approximation(555u128, 1_000_000_000u128)
            .saturating_pow(era_index.saturated_into());

        let staker_payout = inflation_rate.mul_ceil(inflation_decay.mul_ceil(total_tokens));
        let maximum_payout = inflation_rate.mul_ceil(inflation_decay.mul_ceil(total_issuance));

        let staker_to_treasury_ratio = Percent::from_rational_approximation(70u32, 100u32);
        let staker_maximum = staker_to_treasury_ratio.mul_floor(maximum_payout.clone());

        if staker_payout > staker_maximum {
            (staker_maximum, maximum_payout)
        } else {
            (staker_payout, maximum_payout)
        }
    } else if era_index == 360_000 {
        let maximum_payout = 7_777_777_777u128.saturated_into::<N>().saturating_sub(total_issuance);
        let staker_to_treasury_ratio = Percent::from_rational_approximation(70u32, 100u32);
        let staker_maximum = staker_to_treasury_ratio.mul_floor(maximum_payout.clone());
        (staker_maximum, maximum_payout)
    } else {
        // If era > 360,000 no more minting
        let maximum_payout = Zero::zero();
        let staker_payout = Zero::zero();
        (staker_payout, maximum_payout)
    }
}

#[cfg(test)]
mod test {
    // pub const MICROEARTH: u128 = 1_000_000_000;
    // pub const MILLIEARTH: u128 = 1_000 * MICROEARTH;
    // pub const EARTH: u128 = 100 * MILLIEARTH;
    //
	// #[test]
	// fn calculation_is_sensible() {
    //     const TOTAL_TOKENS: u128 = 77_777_777;
    //     const TOTAL_ISSUANCE3: u128 = 77_777_777;
    //
    //     assert_eq!(super::compute_total_payout(0u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17962, 18144));
    //     assert_eq!(super::compute_total_payout(1u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17961, 18143));
    //     assert_eq!(super::compute_total_payout(2u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17961, 18143));
    //     assert_eq!(super::compute_total_payout(3u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17960, 18142));
    //     assert_eq!(super::compute_total_payout(4u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17959, 18141));
    //     assert_eq!(super::compute_total_payout(5u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17958, 18140));
    //     assert_eq!(super::compute_total_payout(6u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17957, 18139));
    //     assert_eq!(super::compute_total_payout(7u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17956, 18138));
    //     assert_eq!(super::compute_total_payout(8u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17955, 18137));
    //     assert_eq!(super::compute_total_payout(9u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17954, 18136));
    //     assert_eq!(super::compute_total_payout(10u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17953, 18135));
    //     assert_eq!(super::compute_total_payout(500u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17519, 17696));
    //     assert_eq!(super::compute_total_payout(1_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (17086, 17259));
    //     assert_eq!(super::compute_total_payout(10_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (10894, 11005));
    //     assert_eq!(super::compute_total_payout(60_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (894, 904));
    //     assert_eq!(super::compute_total_payout(100_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (121, 123));
    //     assert_eq!(super::compute_total_payout(120_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (44, 45));
    //     assert_eq!(super::compute_total_payout(180_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (2, 3));
    //     assert_eq!(super::compute_total_payout(240_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (0, 0));
    //     assert_eq!(super::compute_total_payout(300_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (0, 0));
    //     assert_eq!(super::compute_total_payout(360_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (7623000000, 7700000000));
    //     assert_eq!(super::compute_total_payout(500_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE3), (0, 0));
    //
    //
    //     const TOTAL_ISSUANCE4: u128 = 1_000_000_000;
    //
    //     assert_eq!(super::compute_total_payout(0u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18144, 233278));
    //     assert_eq!(super::compute_total_payout(1u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18143, 233267));
    //     assert_eq!(super::compute_total_payout(2u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18143, 233255));
    //     assert_eq!(super::compute_total_payout(3u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18142, 233244));
    //     assert_eq!(super::compute_total_payout(4u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18141, 233232));
    //     assert_eq!(super::compute_total_payout(5u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18140, 233220));
    //     assert_eq!(super::compute_total_payout(6u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18139, 233209));
    //     assert_eq!(super::compute_total_payout(7u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18138, 233197));
    //     assert_eq!(super::compute_total_payout(8u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18137, 233185));
    //     assert_eq!(super::compute_total_payout(9u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18136, 233174));
    //     assert_eq!(super::compute_total_payout(10u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (18135, 233162));
    //     assert_eq!(super::compute_total_payout(500u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (17696, 227519));
    //     assert_eq!(super::compute_total_payout(1_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (17259, 221901));
    //     assert_eq!(super::compute_total_payout(10_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (11005, 141488));
    //     assert_eq!(super::compute_total_payout(60_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (904, 11612));
    //     assert_eq!(super::compute_total_payout(100_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (123, 1570));
    //     assert_eq!(super::compute_total_payout(120_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (45, 576));
    //     assert_eq!(super::compute_total_payout(180_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (3, 27));
    //     assert_eq!(super::compute_total_payout(240_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (0, 0));
    //     assert_eq!(super::compute_total_payout(300_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (0, 0));
    //     assert_eq!(super::compute_total_payout(360_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (6709999999, 6777777777));
    //     assert_eq!(super::compute_total_payout(500_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE4), (0, 0));
    //
    //
    //     const TOTAL_ISSUANCE5: u128 = 10_000_000_000;
    //
    //     assert_eq!(super::compute_total_payout(0u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18144, 2332780));
    //     assert_eq!(super::compute_total_payout(1u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18143, 2332664));
    //     assert_eq!(super::compute_total_payout(2u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18143, 2332547));
    //     assert_eq!(super::compute_total_payout(3u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18142, 2332431));
    //     assert_eq!(super::compute_total_payout(4u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18141, 2332314));
    //     assert_eq!(super::compute_total_payout(5u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18140, 2332197));
    //     assert_eq!(super::compute_total_payout(6u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18139, 2332081));
    //     assert_eq!(super::compute_total_payout(7u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18138, 2331964));
    //     assert_eq!(super::compute_total_payout(8u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18137, 2331848));
    //     assert_eq!(super::compute_total_payout(9u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18136, 2331731));
    //     assert_eq!(super::compute_total_payout(10u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (18135, 2331614));
    //     assert_eq!(super::compute_total_payout(500u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (17696, 2275182));
    //     assert_eq!(super::compute_total_payout(1_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (17259, 2219006));
    //     assert_eq!(super::compute_total_payout(10_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (11005, 1414876));
    //     assert_eq!(super::compute_total_payout(60_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (904, 116112));
    //     assert_eq!(super::compute_total_payout(100_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (123, 15694));
    //     assert_eq!(super::compute_total_payout(120_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (45, 5759));
    //     assert_eq!(super::compute_total_payout(180_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (3, 266));
    //     assert_eq!(super::compute_total_payout(240_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (0, 0));
    //     assert_eq!(super::compute_total_payout(300_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (0, 0));
    //     assert_eq!(super::compute_total_payout(360_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (0, 0));
    //     assert_eq!(super::compute_total_payout(500_000u32, TOTAL_TOKENS, TOTAL_ISSUANCE5), (0, 0));
    //
    //
    //     const TOTAL_TOKENS2: u128 = 77 * EARTH;
    //     const TOTAL_ISSUANCE6: u128 = 77_777_777 * EARTH;
    //
    //     assert_eq!(super::compute_total_payout(0u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1796240600000, 1814384426300600000));
    //     assert_eq!(super::compute_total_payout(1u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1796150787970, 1814293707079284970));
    //     assert_eq!(super::compute_total_payout(2u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1796060979533, 1814202991486738793));
    //     assert_eq!(super::compute_total_payout(3u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795971174688, 1814112279522961468));
    //     assert_eq!(super::compute_total_payout(4u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795881375232, 1814021573002337422));
    //     assert_eq!(super::compute_total_payout(5u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795791579368, 1813930870110482229));
    //     assert_eq!(super::compute_total_payout(6u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795701788893, 1813840172661780315));
    //     assert_eq!(super::compute_total_payout(7u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795612002010, 1813749478841847253));
    //     assert_eq!(super::compute_total_payout(8u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795522220516, 1813658790465067470));
    //     assert_eq!(super::compute_total_payout(9u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795432442615, 1813568105717056540));
    //     assert_eq!(super::compute_total_payout(10u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1795342670102, 1813477426412198888));
    //     assert_eq!(super::compute_total_payout(500u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1751889711362, 1769585549335648775));
    //     assert_eq!(super::compute_total_payout(1_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1708633888231, 1725892799135500725));
    //     assert_eq!(super::compute_total_payout(10_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (1089454307538, 1100458885498002147));
    //     assert_eq!(super::compute_total_payout(60_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (89405758098, 90308845659873757));
    //     assert_eq!(super::compute_total_payout(100_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (12083628451, 12205685181767592));
    //     assert_eq!(super::compute_total_payout(120_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (4433874426, 4478660991184501));
    //     assert_eq!(super::compute_total_payout(180_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (204078080, 206139472209717));
    //     assert_eq!(super::compute_total_payout(240_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (0, 0));
    //     assert_eq!(super::compute_total_payout(300_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (0, 0));
    //     assert_eq!(super::compute_total_payout(360_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (0, 0));
    //     assert_eq!(super::compute_total_payout(500_000u32, TOTAL_TOKENS2, TOTAL_ISSUANCE6), (0, 0));
    // }
    //
    // #[test]
    // fn maximum_payout_should_grow_predictable() {
    //     const TOTAL_TOKENS: u128 = 77_777_777;
    //     const TOTAL_ISSUANCE: u128 = 77_777_777;
    //
    //     assert_eq!(maximum_payout_after_n_eras(1, TOTAL_TOKENS, TOTAL_ISSUANCE), 77795921); // vs 77795919
    //     assert_eq!(maximum_payout_after_n_eras(10, TOTAL_TOKENS, TOTAL_ISSUANCE), 77959370); // vs 77959356
    //     assert_eq!(maximum_payout_after_n_eras(20, TOTAL_TOKENS, TOTAL_ISSUANCE), 78141297); // vs 78141267
    //     assert_eq!(maximum_payout_after_n_eras(30, TOTAL_TOKENS, TOTAL_ISSUANCE), 78323557); // vs 78323512
    //     assert_eq!(maximum_payout_after_n_eras(40, TOTAL_TOKENS, TOTAL_ISSUANCE), 78506150); // vs 78506091
    //     assert_eq!(maximum_payout_after_n_eras(50, TOTAL_TOKENS, TOTAL_ISSUANCE), 78689077); // vs 78689003
    //     assert_eq!(maximum_payout_after_n_eras(60, TOTAL_TOKENS, TOTAL_ISSUANCE), 78872338); // vs 78872250
    //     assert_eq!(maximum_payout_after_n_eras(70, TOTAL_TOKENS, TOTAL_ISSUANCE), 79055935); // vs 79055831
    //     assert_eq!(maximum_payout_after_n_eras(80, TOTAL_TOKENS, TOTAL_ISSUANCE), 79239866); // vs 79239747
    //     assert_eq!(maximum_payout_after_n_eras(90, TOTAL_TOKENS, TOTAL_ISSUANCE), 79424133); // vs 79423998
    //     assert_eq!(maximum_payout_after_n_eras(100, TOTAL_TOKENS, TOTAL_ISSUANCE), 79608736); // vs 79608584
    // }
    //
    // #[test]
    // fn total_issuance_should_grow_predictable() {
    //     const TOTAL_TOKENS: u128 = 399_999_900 * MICROEARTH;
    //     const TOTAL_ISSUANCE: u128 = 77_777_777 * EARTH;
    //
    //     assert_eq!(total_issuance_after_n_eras(1, TOTAL_TOKENS, TOTAL_ISSUANCE), 7779591991115123927800);
    //     assert_eq!(total_issuance_after_n_eras(2, TOTAL_TOKENS, TOTAL_ISSUANCE), 7781406614728733143145);
    //     assert_eq!(total_issuance_after_n_eras(3, TOTAL_TOKENS, TOTAL_ISSUANCE), 7783221570879691326275);
    //     assert_eq!(total_issuance_after_n_eras(4, TOTAL_TOKENS, TOTAL_ISSUANCE), 7785036859606862127760);
    //     assert_eq!(total_issuance_after_n_eras(5, TOTAL_TOKENS, TOTAL_ISSUANCE), 7786852480950925152130);
    //     assert_eq!(total_issuance_after_n_eras(6, TOTAL_TOKENS, TOTAL_ISSUANCE), 7788668434950745258560);
    //     assert_eq!(total_issuance_after_n_eras(7, TOTAL_TOKENS, TOTAL_ISSUANCE), 7790484721647004106038);
    //     assert_eq!(total_issuance_after_n_eras(8, TOTAL_TOKENS, TOTAL_ISSUANCE), 7792301341078567760186);
    //     assert_eq!(total_issuance_after_n_eras(9, TOTAL_TOKENS, TOTAL_ISSUANCE), 7794118293286119932604);
    //     assert_eq!(total_issuance_after_n_eras(10, TOTAL_TOKENS, TOTAL_ISSUANCE), 7795935578308527893200);
    //     assert_eq!(total_issuance_after_n_eras(11, TOTAL_TOKENS, TOTAL_ISSUANCE), 7797753196186477404336);
    //     assert_eq!(total_issuance_after_n_eras(12, TOTAL_TOKENS, TOTAL_ISSUANCE), 7799571146958836938047);
    //     assert_eq!(total_issuance_after_n_eras(13, TOTAL_TOKENS, TOTAL_ISSUANCE), 7801389430666294305610);
    //     assert_eq!(total_issuance_after_n_eras(14, TOTAL_TOKENS, TOTAL_ISSUANCE), 7803208047347719179031);
    //     assert_eq!(total_issuance_after_n_eras(15, TOTAL_TOKENS, TOTAL_ISSUANCE), 7805026997043801416651);
    //     assert_eq!(total_issuance_after_n_eras(16, TOTAL_TOKENS, TOTAL_ISSUANCE), 7806846279793411888287);
    //     assert_eq!(total_issuance_after_n_eras(17, TOTAL_TOKENS, TOTAL_ISSUANCE), 7808665895637242497501);
    //     assert_eq!(total_issuance_after_n_eras(18, TOTAL_TOKENS, TOTAL_ISSUANCE), 7810485844614165309766);
    //     assert_eq!(total_issuance_after_n_eras(19, TOTAL_TOKENS, TOTAL_ISSUANCE), 7812306126764874272016);
    //     assert_eq!(total_issuance_after_n_eras(20, TOTAL_TOKENS, TOTAL_ISSUANCE), 7814126742128242643221);
    //     assert_eq!(total_issuance_after_n_eras(30, TOTAL_TOKENS, TOTAL_ISSUANCE), 7832351231238604870559);
    //     assert_eq!(total_issuance_after_n_eras(40, TOTAL_TOKENS, TOTAL_ISSUANCE), 7850609085428966439910);
    //     assert_eq!(total_issuance_after_n_eras(50, TOTAL_TOKENS, TOTAL_ISSUANCE), 7868900344496706615256);
    //     assert_eq!(total_issuance_after_n_eras(60, TOTAL_TOKENS, TOTAL_ISSUANCE), 7887225048247182151909);
    //     assert_eq!(total_issuance_after_n_eras(70, TOTAL_TOKENS, TOTAL_ISSUANCE), 7905583236493677458105);
    //     assert_eq!(total_issuance_after_n_eras(80, TOTAL_TOKENS, TOTAL_ISSUANCE), 7923974949057354826702);
    //     assert_eq!(total_issuance_after_n_eras(90, TOTAL_TOKENS, TOTAL_ISSUANCE), 7942400225767204737617);
    //     assert_eq!(total_issuance_after_n_eras(100, TOTAL_TOKENS, TOTAL_ISSUANCE), 7960859106422864852172);
    //
    //
    //     const TOTAL_TOKENS2: u128 = 9_999_900 * MICROEARTH;
    //     const TOTAL_ISSUANCE2: u128 = 93_328_000 * EARTH;
    //
    //     assert_eq!(total_issuance_after_n_eras(1, TOTAL_TOKENS2, TOTAL_ISSUANCE2), 9_334_977_134_585_643_327_800);
    //     assert_eq!(total_issuance_after_n_eras(2, TOTAL_TOKENS2, TOTAL_ISSUANCE2), 9_337_154_668_166_765_363_044);
    //     assert_eq!(total_issuance_after_n_eras(3, TOTAL_TOKENS2, TOTAL_ISSUANCE2), 9_339_332_600_790_002_206_009);
    //     assert_eq!(total_issuance_after_n_eras(4, TOTAL_TOKENS2, TOTAL_ISSUANCE2), 9_341_510_932_501_989_921_367);
    //     assert_eq!(total_issuance_after_n_eras(5, TOTAL_TOKENS2, TOTAL_ISSUANCE2), 9_343_689_663_351_543_703_776);
    //     assert_eq!(total_issuance_after_n_eras(21, TOTAL_TOKENS2, TOTAL_ISSUANCE2), 9_378_603_678_523_154_624_472);
    // }
    //
    // fn maximum_payout_after_n_eras(n: super::EraIndex, total_tokens: u128, total_issuance: u128) -> u128 {
    //     (0..n).fold(total_issuance, |mut acc, era| {
    //         acc += super::compute_total_payout(era, total_tokens, acc).1;
    //         acc
    //     })
    // }
    //
    // fn total_issuance_after_n_eras(n: super::EraIndex, total_tokens: u128, total_issuance: u128) -> u128 {
    //     (0..n).fold(total_issuance, |mut acc, era| {
    //         let (staker_payout, maximum_payout) = super::compute_total_payout(era, total_tokens, acc);
    //         acc += maximum_payout - staker_payout;
    //         acc
    //     })
    // }
}
