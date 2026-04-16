use crate::{
    centered_scaled_world_from_rle, centered_world_from_rle, GrowthFunction, KernelCore,
    KernelMode, LeniaParams, World3D,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SingleSpeciesPreset {
    pub id: &'static str,
    pub official_code: &'static str,
    pub name: &'static str,
    pub source_note: &'static str,
    pub preferred_world_size: usize,
    pub params: LeniaParams,
    pub cells_rle: &'static str,
}

pub fn single_species_presets() -> Vec<SingleSpeciesPreset> {
    vec![
        SingleSpeciesPreset {
            id: "diguttome_saliens",
            official_code: "4Gu2s",
            name: "Diguttome saliens",
            source_note:
                "Official 3D species from Chakazul's animals3D.json using the original RLE seed and ND kernel parameters.",
            preferred_world_size: 64,
            params: official_lenia_params(10, "1,3/4,7/12,11/12", 0.12, 0.01, 1, 1),
            cells_rle: "14.$14.$14.$14.$5.uWyO7.$5.yN3yO5.$4.sE4yO5.$4.rLyH2yOyFqD4.$5.uKxQwRvD5.$14.$14.$14.$14.%14.$14.$6.sAB6.$5.3yOtLrV4.$3.uI6yO4.$3.2yOsMpV3yOrA3.$3.2yOG2.3yO3.$3.yNyOpF2.2yOyN3.$3.xRyOyFrXsE2yOpQ3.$3.tHyK5yO4.$4.BvExOwQuO5.$14.$14.%14.$6.qXpR6.$4.pR4yOqB4.$3.7yOsM3.$2.uVyOuQ3.vW2yOS2.$2.yLyO5.xJyOxA2.$2.yOtL5.D2yO2.$2.yJrU6.2yO2.$2.xGyD5.CyO3.$2.rIyOtA3.PyOyL3.$3.uW2yOwB3yO4.$5.uExOwXpL5.$14.%14.$4.rM3yOwBpB4.$3.7yOrB3.$2.sKyOuN3.wR2yOtQ2.$2.yO7.2yOM.$.yFyO7.wXyOrR.$.yItF8.yOrM.$.xPrW3.R4.yOpR.$.vFwO8.yO2.$2.yOU6.2yO2.$2.rPyOqO3.F2yO3.$3.qU5yOyK4.$5.pTrHpH6.%5.qAqB7.$4.5yOxBM3.$3.2yOuFrUsVxM2yOpQ2.$2.2yO5.vI2yOD.$.2yO7.tPyOwE.$.yO3.rIyOxM3.yOxL.$.yO2.W3yOH2.2yO.$.yO2.pJ3yON2.2yO.$.yO3.qFyJwW3.2yO.$.wSvH8.yOU.$2.xXsA6.yOyK2.$3.yGyOqJF.vNyO4.$4.rMxSyNyIwX5.%5.sQsWqPpN5.$3.vW6yOuA3.$2.2yOvXpD2.uJyLyOvK2.$.vVyOqJ5.sD2yOsK.$.yOsJ3.A3.pB2yO.$.yO3.3yOwH2.yDyO.$.yO2.5yO2.tTyO.$sPyO2.vR4yO2.rTyO.$.yO3.xD3yO2.yMyO.$.yOW3.qGqP3.yOtC.$2.yOB6.2yO2.$3.yOqB4.2yO3.$4.wS4yOwQ4.%4.pLuBsVuSvOpT4.$3.7yOyKK2.$2.2yOwAB.AuMyL2yOD.$.qUyOsW5.pQ2yOxH.$.yO4.sXpG3.2yOA$pLyO2.rN4yO2.xKyOV$xXwJ2.2yOqGsPyO2.sPyO.$xOxD2.yG2yOyByO2.rAyO.$.yO2.sQ4yO2.2yO.$.yOA2.qNuGtP3.yOuG.$.tSyOA6.2yO2.$2.qGyOpX4.2yO3.$3.qHxW4yOyI4.%4.HsLtKwPvWpC4.$3.wJ6yOtJ3.$2.sTyOyBsCpQrPvU2yOyM2.$2.yOvG5.W2yOsS.$.yOwE8.2yO.$rNyO3.3yOqR2.xMyOH$sOyO2.xB4yO2.uPyO.$AyO2.wQ4yO2.vEyO.$.yO3.3yOsN2.2yO.$.yOpR3.rN4.yOtO.$.pUyOR6.2yO2.$3.yOvO3.A2yO3.$4.wT4yOyA4.%5.qCrHsCqA5.$4.5yOxTpD3.$3.2yOyNxLwTyM2yOsN2.$2.2yO5.vG2yOpA.$.2yO7.wHyOuD.$.yOpE3.A3.qC2yO.$.yO3.qByOxI3.2yO.$.yO3.sM2yO3.2yO.$.yOL8.yOwW.$.wQyO7.2yOW.$2.xFyO5.vDyO3.$3.xUyOuXUH2yO4.$5.vHyFyKxR5.%14.$4.sB4yOsG4.$3.xF7yOC2.$2.3yOyHwRxIyN3yO2.$2.2yO5.uG2yOV.$.qTyOqT5.F2yOvD.$.yNyO7.2yOtX.$.2yO7.yKyOtA.$.wByO7.2yOqC.$2.2yO5.rPyOxX2.$3.2yOM2.xD2yO3.$4.6yO4.$6.qX7.%14.$5.UqLqJpS5.$4.rN4yOuQQ3.$3.qX6yOuKW2.$3.2yOuDsGsXxI2yOrS2.$2.vXyOqSE2.sAyByOuS2.$2.2yO5.vWyOtS2.$2.2yO5.vUyOsU2.$2.vHyO4.sAyHyOqP2.$3.2yO2.pW2yOwV3.$3.sB6yO4.$5.qMyFqX6.$14.%14.$14.$5.SqOqLpI5.$4.pL2yOxDxRrG4.$3.C6yOK3.$3.7yOtF3.$3.7yOuA3.$3.7yOsF3.$3.yM6yO4.$4.5yOsC4.$5.pM2yO6.$14.$14.%14.$14.$14.$5.BMC6.$5.qFqPqNqT5.$4.KwI2yOuSpX4.$5.3yOuKpN4.$5.yJ2yOvUpO4.$5.pTsDtDuI5.$7.D6.$14.$14.$14.!",
        },
        SingleSpeciesPreset {
            id: "diguttome_tardus",
            official_code: "3Gu2t",
            name: "Diguttome tardus",
            source_note:
                "Official 3D species from Chakazul's animals3D.json using the original RLE seed and ND kernel parameters.",
            preferred_world_size: 64,
            params: official_lenia_params(10, "2/3,1,5/6", 0.15, 0.016, 1, 1),
            cells_rle: "13.$13.$13.$13.$13.$5.PuV6.$5.pEuC6.$13.$13.$13.$13.$13.%13.$13.$13.$4.sM3yO5.$3.pG5yO4.$3.xI5yOqO3.$3.xM5yOyE3.$3.qJ5yO4.$4.sP3yOqL4.$13.$13.$13.%13.$13.$4.5yO4.$3.7yO3.$2.wQ2yOpM.3yOJ2.$2.2yOqD3.3yO2.$2.2yOF3.3yO2.$2.xC2yO2.rN2yO3.$3.7yO3.$4.4yOyF4.$6.O6.$13.%13.$4.4yOqV4.$3.7yO3.$2.2yOvN2.xW3yO2.$2.yOtW4.tA2yO2.$.wQyO6.2yOU.$.xEyO6.2yOL.$.qLyOpN5.2yO2.$2.2yOA3.3yO2.$3.7yO3.$4.xR3yOwB4.$13.%5.PtNM5.$3.6yOqO3.$2.4yOxC3yOuF2.$.rXyOuU4.tC2yOA.$.2yO6.yM2yO.$.yOpH.uG2yOqN2.2yO.$.yOE.yJ3yO2.2yO.$.2yO2.2yO2.P2yO.$.tGyOH5.2yO2.$2.2yOqH3.3yO2.$3.xJ5yOxT3.$5.rEvMqB5.%4.E3yOK4.$3.7yO3.$2.3yO2.rW3yO2.$.2yO6.2yOuQ.$.yOV2.2yOpF2.2yO.$.yO2.yO2.yO2.2yO.$pRyO2.yO2.yO2.2yO.$.yO2.4yO2.2yO.$.2yO2.xByG2.2yOyM.$2.2yO4.vF2yO2.$3.7yO3.$4.rD3yOqA4.%4.uE3yOqL4.$3.7yO3.$2.2yOvI2.G3yO2.$.2yO6.3yO.$.yO3.2yOsW2.2yO.$tAyO2.yO2.xS2.2yOA$tSyO2.yO2.vCpF.2yO.$.yO2.4yO2.2yO.$.2yO2.yJyO2.wX2yO.$2.yOwX4.uL2yO2.$2.pQ3yOyN3yO3.$4.vA3yOwE4.%4.O2yOxGK4.$3.7yO3.$2.3yOpM.yE3yO2.$.2yOtR5.2yOsW.$.2yO2.rGtM2.sF2yO.$.yO2.2yOxQwS2.2yO.$.yO2.2yOuKyO2.2yO.$.yOpV2.2yOpD2.2yO.$.yNyO6.2yOrN.$2.2yO4.3yO2.$3.7yO3.$5.3yO5.%5.GqBJ5.$3.sF5yOqA3.$2.8yOrE2.$2.2yOR3.3yO2.$.2yO6.2yOvM.$.2yO6.xD2yO.$.2yO2.EqG2.wB2yO.$.2yO6.2yOuI.$2.2yO4.wH2yO2.$2.xO2yOAO3yOqL2.$3.uM5yOC3.$6.vL6.%13.$4.qC3yOqE4.$3.6yOxQ3.$2.8yOtG2.$2.2yOxW2.tA3yO2.$.xL2yO4.xJ2yOJ.$.wI2yO4.vS2yOF.$2.2yO4.3yO2.$2.3yOyIvF3yOuM2.$3.7yO3.$4.uP3yOB4.$13.%13.$13.$4.tQ3yOrQ4.$3.6yOtT3.$3.7yOC2.$2.8yOsC2.$2.8yOrP2.$2.sW7yO3.$3.6yOtL3.$4.4yOpM4.$13.$13.%13.$13.$13.$5.tIyOsF5.$4.4yOvM4.$3.A5yOH3.$4.5yOB3.$4.4yOvB4.$5.2yOrQ5.$13.$13.$13.!",
        },
        SingleSpeciesPreset {
            id: "triguttome_labens",
            official_code: "4Gu3l",
            name: "Triguttome labens",
            source_note:
                "Official 3D species from Chakazul's animals3D.json using the original RLE seed and ND kernel parameters.",
            preferred_world_size: 64,
            params: official_lenia_params(10, "1,5/12,1/12,1/6", 0.16, 0.015, 1, 1),
            cells_rle: "11.$11.$11.$11.$4.qXyDxF4.$4.xT2yO4.$4.yB2yOvT3.$4.sPxDuB4.$11.$11.$11.$11.%11.$11.$5.pK5.$3.vX4yO3.$2.qM5yOuW2.$2.yNyOB.3yO2.$2.yOvO2.A2yO2.$2.yByOpUK3yO2.$3.5yO3.$4.rHuSpX4.$11.$11.%11.$11.$3.5yO3.$2.3yOyE3yO2.$.sQyO4.2yOtD.$.yOpR.yNyO2.2yO.$.yODwD2yO2.2yO.$.yBsVtPyCyO2.2yO.$2.yOpU3.2yO2.$2.pN5yOsS2.$4.qLsT5.$11.%11.$4.3yOD3.$2.7yO2.$.vLyO4.xXyOwM.$.yO.wX2yOqO.2yO.$WxX.yOrXJyO2.yOpK$tEtDxDyO2.yOrM.yOrF$.yOqX2yOsVyO.DyO.$.yOLuW2yOwT.2yO.$2.yOqN3.2yO2.$3.yA3yOxU3.$11.%11.$3.5yO3.$.rN2yOrHVxN2yO2.$.yOE.QqW2.2yO.$sOyO.2yO.yOLKyOrA$yN.yOI3.yO.2yO$yO.yO.S2.yO.2yO$xTDyOsN3.yO.2yO$.yOqE2yOsLyO.wEyO.$.vQyOpSwKxD2.2yO.$2.uLyOxEvM3yO2.$5.qQ5.%11.$2.pD5yOA2.$.3yO2.rK2yOpV.$.yO2.rLyED.2yO.$xVpK.yO2.uMrK.yOwL$yO.yO4.xJ.yLyO$yO.yO.2yO.vE.xGyO$yO.yOH3.yO.2yO$qBwPsQyOpP.yOA.yOS$.yHqL.yFyO2.2yO.$2.xLyOrFA3yO2.$4.qTxCwJ4.%11.$3.5yO3.$.pG2yOvHsUxJ2yOA.$.2yO2.pP.qH2yO.$wCyO.yOxLrKyOXtDyOrH$yO.2yO3.xL.2yO$yO.yO4.yO.2yO$wOFyAwL3.vQ.2yO$.yO.4yO.wByO.$.uQyO.qA2.wR2yO.$2.sP6yO2.$5.vSvL4.%11.$3.sG3yOV3.$2.7yO2.$.2yOpU3.yByOwW.$.yO2.tCvMP.2yO.$uWyO.2yOwOyOCtVyOqO$tExD.yOtI.yOVJyOtG$.yO.4yO.vDyOB$.yOpN.yKqS2.2yO.$2.yOsR2.qS2yO2.$3.yN4yO3.$11.%11.$5.V5.$3.5yO3.$2.3yOxJ3yO2.$.2yOtG2.DyLyOuU.$.2yO4.rU2yO.$.yO2.KqE2.2yO.$.2yO4.tD2yO.$.GyOpJ2.qV2yO2.$2.tJ5yOsR2.$4.yI2yO4.$11.%11.$11.$4.pJtOP4.$3.5yO3.$2.6yOvV2.$2.2yOxXvX3yO2.$.qA2yOvSrIyE2yO2.$2.3yOyH3yO2.$2.vC5yOH2.$3.rU3yO4.$11.$11.%11.$11.$11.$11.$4.2yOwG4.$3.4yOsR3.$3.4yOvN3.$3.uP3yOL3.$4.MuM5.$11.$11.$11.!",
        },
        SingleSpeciesPreset {
            id: "sphaerome_lithos",
            official_code: "1Sp1l",
            name: "Sphaerome lithos",
            source_note:
                "Official 3D species from Chakazul's animals3D.json using the original RLE seed and ND kernel parameters.",
            preferred_world_size: 64,
            params: official_lenia_params(12, "1", 0.14, 0.016, 1, 1),
            cells_rle: "11.$11.$11.$4.3yOxH3.$3.5yO3.$2.rI6yO2.$2.rS6yO2.$3.5yO3.$4.3yOyF3.$11.$11.$11.%11.$11.$3.5yO3.$2.7yO2.$2.8yO.$.4yOsV4yO.$.4yOsA4yO.$2.8yO.$2.7yO2.$3.5yO3.$11.$11.%11.$3.sX4yO3.$2.7yO2.$.4yO.4yO.$.2yOwT4.2yO.$.2yO5.3yO$.2yO5.3yO$.2yOwB4.2yO.$.4yO.4yO.$2.7yO2.$3.vJ4yO3.$11.%11.$3.6yO2.$.rU8yO.$.2yO5.2yO.$xE2yO6.2yO$2yO7.2yO$2yO7.2yO$xO2yO6.2yO$.2yO5.2yO.$.tP8yO.$3.6yO2.$11.%4.3yO4.$2.7yO2.$.3yO3.3yO.$.2yO5.3yO$2yO7.2yO$2yO7.2yO$2yO7.2yO$2yO7.2yO$.2yO5.3yO$.3yO3.3yO.$2.7yO2.$4.3yO4.%4.3yO4.$2.7yO2.$.3yO3.3yO.$.2yO5.3yO$2yO7.2yO$2yO7.2yO$2yO7.2yO$2yO7.2yO$.2yO5.uK2yO$.3yO3.3yO.$2.7yO2.$4.3yO4.%5.yOuN4.$2.7yO2.$.4yO.uU3yO.$.2yO5.3yO$2yO7.2yO$2yO7.2yO$2yO7.2yO$2yO7.2yO$.2yO5.3yO$.4yO.F3yO.$2.7yO2.$4.pR2yO4.%11.$3.5yO3.$2.8yO.$.3yO3.3yO.$.2yO5.3yO$2yOsQ6.2yO$2yOrS6.2yO$.2yO5.3yO$.3yO3.sS2yO.$2.8yO.$3.5yO3.$11.%11.$4.3yO4.$2.7yO2.$2.8yO.$.3yO3.3yO.$.2yOuX4.2yO.$.2yOtS4.2yO.$.3yO3.3yO.$2.8yO.$2.7yO2.$4.3yO4.$11.%11.$11.$4.3yO4.$3.6yO2.$2.7yO2.$2.7yOtI.$2.7yOuR.$2.7yO2.$3.6yO2.$4.3yO4.$11.$11.%11.$11.$11.$11.$4.3yO4.$3.5yO3.$3.5yO3.$4.3yOrF3.$11.$11.$11.$11.!",
        },
        SingleSpeciesPreset {
            id: "disphaerome_lithos",
            official_code: "3Sp2l",
            name: "Disphaerome lithos",
            source_note:
                "Official 3D species from Chakazul's animals3D.json using the original RLE seed and ND kernel parameters.",
            preferred_world_size: 64,
            params: official_lenia_params(12, "1,1,1", 0.15, 0.017, 1, 1),
            cells_rle: "16.$16.$16.$16.$6.xXyOwCB6.$5.5yOsN5.$5.6yO5.$4.pN6yOpL4.$5.6yO5.$5.wH3yOyGpB5.$6.uByOsK7.$16.$16.$16.$16.%16.$16.$7.qD8.$5.6yO5.$4.8yO4.$3.wS8yO4.$3.10yO3.$3.10yO3.$3.10yO3.$3.uK8yO4.$4.8yO4.$5.6yO5.$16.$16.$16.%16.$7.H8.$4.rG6yO5.$3.vB8yO4.$3.10yO3.$2.5yOtRsO4yO3.$2.4yO4.4yO2.$2.3yOxS4.4yO2.$2.4yO4.4yO2.$2.4yOyD2.4yOF2.$3.10yO3.$3.wX8yO4.$5.6yO5.$16.$16.%16.$5.6yO5.$3.9yO4.$2.qM10yO3.$2.3yOyA4.4yO2.$.O2yOyI6.3yO2.$.3yO8.2yO2.$.3yO8.2yOrL.$.3yO8.2yOE.$.M2yOxK6.3yO2.$2.3yO5.tE3yO2.$2.sS10yO3.$3.xN8yOpU3.$5.6yO5.$16.%6.sXwAtE7.$4.8yO4.$3.10yO3.$2.3yOxK4.4yO2.$.uR2yOR6.3yO2.$.3yO8.3yO.$.2yO9.3yO.$.2yO9.K2yO.$.2yO9.3yO.$.3yO8.2yOwW.$.tV2yO7.wH2yO2.$2.3yOpR4.4yO2.$3.10yO3.$4.8yO4.$6.pCqL8.%5.yI4yOrH5.$3.xH8yO4.$2.5yOsTrR4yOpS2.$.rW2yOxI6.3yO2.$.3yO8.3yO.$.2yO9.D2yO.$tT2yO3.4yO3.2yO.$vM2yO3.4yO3.2yO.$qC2yO3.4yO3.2yO.$.2yO4.yOuD4.2yO.$.3yO8.3yO.$.sC2yOrQ6.3yO2.$2.4yOI2.sN3yOsQ2.$3.yI8yO4.$5.xC4yO6.%5.6yO5.$3.10yO3.$2.4yO4.4yO2.$.3yO7.tK2yOsN.$.2yO9.3yO.$uM2yO3.4yO3.2yO.$3yO2.2yO2.yOsO2.2yOsR$2yOqT2.yO3.2yO2.2yOrN$3yO2.2yO2.yOvX2.2yO.$qK2yO3.4yO3.2yO.$.2yO10.2yO.$.3yO8.2yOpT.$2.3yOtD4.4yO2.$3.10yO3.$5.6yO5.%4.B6yO5.$3.10yO3.$2.3yOyB4.4yO2.$.3yO8.2yOwA.$.2yO9.sF2yO.$wO2yO3.4yO3.2yO.$2yOqK2.yOpL2.2yO2.2yOsF$2yO3.yO4.yO2.2yOpX$2yO3.yO3.uNyO2.2yO.$sS2yO2.D4yO3.2yO.$.2yO4.xKvB4.2yO.$.3yO8.2yOwI.$2.3yO5.sN3yO2.$3.10yO3.$5.6yO5.%5.6yO5.$3.10yO3.$2.4yO4.4yO2.$.3yO7.pJ2yOpU.$.2yO9.3yO.$rH2yO3.4yO3.2yO.$3yO2.2yO2.yOvH2.2yO.$2yO3.yO3.uUyO2.2yO.$2yOrM2.2yO2.yOwI2.2yO.$.2yO3.4yO3.2yO.$.2yO10.2yO.$.3yO8.2yO2.$2.3yOpH4.4yO2.$3.9yOyJ3.$5.6yO5.%5.wK3yOtL6.$3.vF8yO4.$2.5yO2.4yOP2.$.qN2yOyJ6.3yO2.$.3yO8.2yOxD.$.2yO4.yIuE4.2yO.$pH2yO3.4yO3.2yO.$rC2yO2.tL4yO3.2yO.$.2yO3.4yO3.2yO.$.2yO3.uJyOwI4.2yO.$.2yOvE8.2yOyB.$2.2yO7.wT2yO2.$2.4yO4.3yO3.$3.sC8yO4.$5.qC4yO6.%6.rIsF8.$4.8yO4.$3.10yO3.$2.3yOrR4.4yO2.$.vB2yO7.3yO2.$.3yO8.3yO.$.2yO9.D2yO.$.2yO4.xCtT4.2yO.$.2yO10.2yO.$.2yOxB8.2yOxE.$.sW2yO8.2yO2.$2.3yO5.I3yO2.$3.10yO3.$4.yL6yOxH4.$7.tQ8.%16.$5.6yO5.$3.9yO4.$2.M10yO3.$2.3yOtN4.rA3yO2.$.qK2yOtV6.3yO2.$.3yO8.2yOtS.$.3yO8.2yOwE.$.3yO8.2yO2.$2.2yO7.yM2yO2.$2.3yO5.pT3yO2.$2.wM4yOwUwR4yO3.$3.vR8yO4.$5.5yOvT5.$16.%16.$16.$5.6yO5.$3.Q9yO3.$3.3yO4.4yO2.$2.3yO6.3yO2.$2.2yO2.4yO2.3yO.$2.2yO2.xD3yO2.3yO.$2.3yO2.3yO2.yF2yO.$2.2yO3.3yO2.3yO.$2.2yO8.3yO.$2.4yO5.3yO2.$3.3yO.uH6yO2.$4.9yO3.$5.6yO5.$16.$16.%16.$16.$16.$5.6yO5.$4.8yO4.$3.10yO3.$3.4yO2.5yO2.$3.3yOqT3.4yO2.$3.3yO.2yO.xE3yO2.$3.3yOPyOwV.4yO2.$3.5yOvX5yO2.$3.10yO3.$4.8yO4.$5.6yO5.$16.$16.$16.%16.$16.$16.$16.$6.4yO6.$5.6yO5.$4.8yO4.$4.9yO3.$4.9yO3.$4.9yO3.$4.8yO4.$5.6yO5.$6.4yO6.$16.$16.$16.$16.!",
        },
        SingleSpeciesPreset {
            id: "ovome_limus",
            official_code: "4Ov1l",
            name: "Ovome limus",
            source_note:
                "Official 3D species from Chakazul's animals3D.json using the original RLE seed and ND kernel parameters.",
            preferred_world_size: 64,
            params: official_lenia_params(10, "7/12,1/6,1,5/6", 0.15, 0.019, 1, 1),
            cells_rle: "13.$13.$13.$13.$6.vX6.$6.yO6.$6.xK6.$13.$13.$13.$13.$13.$13.%13.$13.$5.yFyO6.$3.yL5yO4.$3.7yO3.$3.7yO3.$3.7yO3.$3.7yO3.$3.6yO4.$5.3yO5.$13.$13.$13.%13.$5.xXyOwD5.$3.6yO4.$2.8yO3.$2.9yO2.$2.9yO2.$2.9yO2.$2.8yOwS2.$2.yK7yO3.$3.7yO3.$5.rF2yO5.$13.$13.%5.yGyOvV5.$3.7yO3.$2.8yO3.$.4yO3.3yO2.$.3yO4.vP3yO.$.3yO5.3yO.$.3yO5.3yO.$.4yOD2.4yO.$.10yOH.$2.9yO2.$3.7yO3.$5.3yO5.$13.%3.yK5yO4.$2.8yO3.$.4yO3.3yO2.$.2yO6.3yO.$3yO2.rOyO3.2yO.$3yO2.3yO2.2yO.$3yO2.3yO2.2yO.$3yO2.rHyOA.3yO.$.3yO5.3yO.$.10yOF.$2.yC7yO3.$3.vR5yO4.$13.%3.7yO3.$2.9yO2.$.3yO5.2yOqG.$yN2yO3.yO3.2yO.$2yO2.yO2.yO2.2yO.$2yO2.yO3.yO2.yOwQ$2yO2.yO3.yO.3yO$2yOxL.4yOC.2yOvD$3yO2.vVyO2.3yO.$.3yOS3.sQ3yO.$2.8yOxT2.$3.6yOyM3.$13.%3.7yO3.$2.3yO2.sP3yO2.$.2yO6.3yO.$2yO8.2yO.$2yO2.J6.2yO$2yO.qX4.yO2.2yO$2yO.yO4.yO2.2yO$2yO2.yO2.tUyO.3yO$3yO.S3yO2.2yO.$.3yO5.3yO.$2.9yO2.$3.7yO3.$6.yO6.%3.7yO3.$2.3yO3.3yO2.$.2yO6.yI2yO.$2yO8.2yO.$2yO9.2yO$2yO.yO4.wA2.2yO$2yO.yO4.yO2.2yO$2yO2.yO3.yO.pHyOqC$3yO2.2yOpE2.2yO.$.3yO5.3yO.$2.3yOpT5yO2.$3.7yO3.$6.yOpR5.%3.6yO4.$2.3yO.wW3yOxP2.$.2yO6.2yOxB.$3yO7.2yO.$2yO8.2yOxD$2yO9.yOwR$2yO9.yOsX$2yO5.xI2.2yO.$3yO3.tE3.2yO.$.3yO4.qP3yO.$2.9yO2.$3.7yO3.$6.pL6.%4.5yO4.$2.8yO3.$.3yO5.2yO2.$.2yO7.2yO.$3yO7.2yO.$2yO8.2yO.$2yO8.2yO.$uQ2yO7.2yO.$.2yO6.3yO.$.4yO2.D3yO2.$2.8yO3.$3.6yO4.$13.%13.$3.6yO4.$2.8yOxN2.$.3yO5.2yO2.$.2yO6.2yO2.$.2yO7.2yO.$.2yO6.3yO.$.3yO5.2yO2.$.4yO3.3yO2.$2.8yOsV2.$3.6yO4.$5.tPyO6.$13.%13.$5.2yO6.$3.6yO4.$2.xL7yO3.$2.3yO2.3yOuU2.$2.3yO3.3yO2.$2.3yO3.3yO2.$2.9yO2.$2.wJ7yO3.$3.6yOyG3.$6.2yO5.$13.$13.%13.$13.$13.$4.5yO4.$3.6yO4.$3.7yO3.$3.7yO3.$3.7yO3.$4.5yO4.$5.vR2yO5.$13.$13.$13.!",
        },
    ]
}

pub fn seeded_world_for_preset(
    world_shape: (usize, usize, usize),
    preset: &SingleSpeciesPreset,
) -> World3D {
    centered_world_from_rle(world_shape, preset.cells_rle)
}

pub fn scaled_seed_shape_for_preset(
    preset: &SingleSpeciesPreset,
    scale_factor: usize,
) -> (usize, usize, usize) {
    let scale_factor = scale_factor.max(1);
    let (depth, height, width) = crate::decode_lenia_rle_3d(preset.cells_rle).dim();
    (
        depth * scale_factor,
        height * scale_factor,
        width * scale_factor,
    )
}

pub fn scaled_params_for_preset(preset: &SingleSpeciesPreset, scale_factor: usize) -> LeniaParams {
    let mut params = preset.params.clone();
    params.radius_cells = params.radius_cells.saturating_mul(scale_factor.max(1));
    params
}

pub fn seeded_world_for_preset_scaled(
    world_shape: (usize, usize, usize),
    preset: &SingleSpeciesPreset,
    scale_factor: usize,
) -> World3D {
    centered_scaled_world_from_rle(world_shape, preset.cells_rle, scale_factor)
}

fn official_lenia_params(
    radius_cells: usize,
    bands: &'static str,
    mu: f32,
    sigma: f32,
    kernel_index: usize,
    growth_index: usize,
) -> LeniaParams {
    LeniaParams {
        kernel_mode: KernelMode::LeniaBands,
        kernel_core: kernel_core_from_official_index(kernel_index),
        radius_cells,
        mu,
        sigma,
        time_step: 0.1,
        growth_function: growth_from_official_index(growth_index),
        shells: vec![],
        bands: parse_fraction_list(bands),
        mace_beta: None,
    }
}

fn parse_fraction_list(input: &str) -> Vec<f32> {
    input
        .split(',')
        .filter(|part| !part.is_empty())
        .map(parse_fraction)
        .collect()
}

fn parse_fraction(input: &str) -> f32 {
    if let Some((numerator, denominator)) = input.split_once('/') {
        let numerator = numerator.parse::<f32>().expect("valid fraction numerator");
        let denominator = denominator
            .parse::<f32>()
            .expect("valid fraction denominator");
        numerator / denominator
    } else {
        input.parse::<f32>().expect("valid decimal fraction")
    }
}

fn kernel_core_from_official_index(index: usize) -> KernelCore {
    match index {
        1 => KernelCore::Polynomial,
        2 => KernelCore::Exponential,
        3 => KernelCore::Step,
        4 => KernelCore::Staircase,
        _ => KernelCore::Polynomial,
    }
}

fn growth_from_official_index(index: usize) -> GrowthFunction {
    match index {
        1 => GrowthFunction::Polynomial,
        2 => GrowthFunction::Exponential,
        3 => GrowthFunction::Step,
        _ => GrowthFunction::Polynomial,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        scaled_params_for_preset, scaled_seed_shape_for_preset, seeded_world_for_preset,
        seeded_world_for_preset_scaled, single_species_presets,
    };
    use crate::{FftBackend, KernelMode, SimulationBackend};

    #[test]
    fn species_library_is_non_empty() {
        assert!(!single_species_presets().is_empty());
    }

    #[test]
    fn official_species_use_lenia_bands() {
        for preset in single_species_presets() {
            assert_eq!(preset.params.kernel_mode, KernelMode::LeniaBands);
        }
    }

    #[test]
    fn seeded_world_contains_life() {
        let presets = single_species_presets();
        let world = seeded_world_for_preset((64, 64, 64), &presets[0]);
        assert!(world.view().iter().copied().fold(0.0_f32, f32::max) > 0.0);
    }

    #[test]
    fn official_species_stay_localized_in_early_steps() {
        for preset in single_species_presets() {
            let mut backend = FftBackend::new();
            let size = preset.preferred_world_size;
            let mut world = seeded_world_for_preset((size, size, size), &preset);

            for _ in 0..8 {
                world = backend.step(&world, &preset.params);
            }

            let min = world.view().iter().copied().fold(1.0_f32, f32::min);
            let mean = world.mean();
            assert!(
                min < 1.0e-3 && mean < 0.2,
                "{} lost locality in the early probe: min={min}, mean={mean}",
                preset.id
            );
        }
    }

    #[test]
    fn scaled_species_seed_and_params_grow_together() {
        let preset = &single_species_presets()[0];
        let params = scaled_params_for_preset(preset, 2);
        let world = seeded_world_for_preset_scaled((128, 128, 128), preset, 2);

        assert_eq!(params.radius_cells, preset.params.radius_cells * 2);
        assert!(world.mean() > 0.0);
    }

    #[test]
    fn scaled_seed_shape_grows_linearly_with_scale() {
        let preset = &single_species_presets()[0];
        let base = scaled_seed_shape_for_preset(preset, 1);
        let scaled = scaled_seed_shape_for_preset(preset, 3);

        assert_eq!(scaled.0, base.0 * 3);
        assert_eq!(scaled.1, base.1 * 3);
        assert_eq!(scaled.2, base.2 * 3);
    }
}
