use std::fmt::Debug;
use std::marker::PhantomData;

use crate::{commit, scalar, eval};

use crate::schema::ast::{
    SchemaItem,
    CommitQuery,
};

use crate::schema::utils::{
    RingUtils,
};


use crate::arith::api::{
    ContextGroup,
    ContextRing,
    PowConstant,
};

use crate::schema::{
    EvaluationProof,
    SchemaGenerator,
};

use crate::{arith_in_ctx, infix2postfix};

pub struct ParamsPreprocessed<'a, P> {
    q_m: &'a P,
    q_l: &'a P,
    q_r: &'a P,
    q_o: &'a P,
    q_c: &'a P,
    sigma1: &'a P,
    sigma2: &'a P,
    sigma3: &'a P,
}

pub struct VerifyCommitments<'a, P> {
    a: &'a P,
    b: &'a P,
    c: &'a P,
    z: &'a P,
    tl: &'a P,
    tm: &'a P,
    th: &'a P,
    w_z: &'a P,
    w_zw: &'a P,

}

pub struct VerifyEvals<'a, S> {
    a_xi: &'a S,
    b_xi: &'a S,
    c_xi: &'a S,
    sigma1_xi: &'a S,
    sigma2_xi: &'a S,
    z_xiw: &'a S,
    zh_xi: Option<S>,
    l1_xi: Option<S>,
    pi_xi: Option<S>,
}

pub struct PlonkCommonSetup<'a, S> {
    l: u32,
    n: u32,
    k: Vec<&'a S>,
    w: &'a S, //TODO the unit root of 2^n = 1
    one: &'a S,
    zero: &'a S,
}


pub struct PlonkVerifierParams <
    'a, C, S, P, Error:Debug,
    SGate: ContextGroup<C, S, S, Error> + ContextRing<C, S, S, Error>,
    PGate: ContextGroup<C, S, P, Error>,
> {
    //public_wit: Vec<C::ScalarExt>,
    common: PlonkCommonSetup<'a, S>,
    params: ParamsPreprocessed<'a, P>,
    commits: VerifyCommitments<'a, P>,
    evals: VerifyEvals<'a, S>,
    beta: &'a S,
    gamma: &'a S,
    alpha: &'a S,
    u: &'a S,
    v: &'a S,
    xi: &'a S,
    xi_n: &'a S,
    sgate: &'a SGate,
    pgate: &'a PGate,
    _ctx: PhantomData<C>,
    _error: PhantomData<Error>
}

impl<'a, C, S:Clone, P:Clone, Error:Debug, SGate: ContextGroup<C, S, S, Error> + ContextRing<C, S, S, Error>, PGate:ContextGroup<C, S, P, Error>>
    PlonkVerifierParams<'a, C, S, P, Error, SGate, PGate> {

    fn get_common_evals(
        &self,
        ctx: &mut C,
    ) -> Result<[SchemaItem<S, P>; 6], Error> {
        let sgate = self.sgate;
        let n = &sgate.from_constant(self.common.n)?;
        let one = self.sgate.one();
        let xi = self.xi;

        let xi_n = &self.sgate.pow_constant(ctx, self.xi, self.common.n)?;
        let xi_2n = &self.sgate.pow_constant(ctx, xi_n, 2)?;

        // zh_xi = xi ^ n - 1
        let zh_xi = &self.sgate.minus(ctx, xi_n, self.sgate.one())?;

        // l1_xi = w * (xi ^ n - 1) / (n * (xi - w))
        let l1_xi = &{
            let w = self.common.w;
            arith_in_ctx!([sgate, ctx] w * (xi_n - one) / (n * (xi - w)))
        }?;

        let pi_xi = &{
            let w_vec = sgate.pow_constant_vec(ctx, self.common.w, self.common.l)?;
            let mut pi_vec = vec![];
            for i in 0..self.common.l {
                let wi = &w_vec[i as usize];
                // li_xi = (w ^ i) * (xi ^ n - 1) / (n * (xi - w ^ i))
                let li_xi = arith_in_ctx!([sgate, ctx] wi * (xi_n - one) / (n * (xi - wi))).unwrap();
                pi_vec.push(li_xi);
            }

            let mut pi_xi = (&pi_vec)[0].clone();
            for i in 1..self.common.l {
                let next = &(pi_vec)[i as usize];
                let curr = &pi_xi;
                pi_xi = arith_in_ctx!([sgate, ctx] curr + next).unwrap()
            }
            pi_xi.clone()
        };
        Ok([scalar!(xi), scalar!(xi_n), scalar!(xi_2n), scalar!(zh_xi), scalar!(l1_xi), scalar!(pi_xi)])
    }

    fn get_proof_xi (
        &self,
        ctx: &mut C,
    ) -> Result<EvaluationProof<S, P>, Error> {
        let zero = self.sgate.zero();
        let one = self.sgate.one();
        let a = CommitQuery{c: Some(self.commits.a), v: Some(self.evals.a_xi)};
        let b = CommitQuery{c: Some(self.commits.b), v: Some(self.evals.b_xi)};
        let c = CommitQuery{c: Some(self.commits.c), v: Some(self.evals.c_xi)};
        let qm = CommitQuery::<S, P> {c: Some(self.params.q_m), v: None};
        let ql = CommitQuery::<S, P> {c: Some(self.params.q_l), v: None};
        let qr = CommitQuery::<S, P> {c: Some(self.params.q_r), v: None};
        let qo = CommitQuery::<S, P> {c: Some(self.params.q_o), v: None};
        let qc = CommitQuery::<S, P> {c: Some(self.params.q_c), v: None};
        let z = CommitQuery::<S, P> {c: Some(self.commits.z), v: None};
        let zxi = CommitQuery::<S, P> {c: Some(self.commits.z), v: Some(self.evals.z_xiw)};
        let sigma1 = CommitQuery::<S, P> {c: None, v: Some(self.evals.sigma1_xi)};
        let sigma2 = CommitQuery::<S, P> {c: None, v: Some(self.evals.sigma2_xi)};
        let sigma3 = CommitQuery::<S, P> {c: Some(self.params.sigma3), v: None};
        let tl = CommitQuery::<S, P> {c: Some(self.commits.tl), v: None};
        let tm = CommitQuery::<S, P> {c: Some(self.commits.tm), v: None};
        let th = CommitQuery::<S, P> {c: Some(self.commits.th), v: None};
        let [xi, xi_n, xi_2n, zh_xi, l1_xi, pi_xi] = self.get_common_evals(ctx)?;
        let neg_one = &(self.sgate.minus(ctx, zero, one)?);
        let r = eval!(a) * eval!(b) * commit!(qm) + eval!(a) * commit!(ql)
            + eval!(b) * commit!(qr) + eval!(c) * commit!(qo) + pi_xi + commit!(qc)
            + scalar!(self.alpha) * (
                  (eval!(a) + (scalar!(self.beta) * xi.clone()) + scalar!(self.gamma))
                * (eval!(b) + (scalar!(self.beta) * xi.clone()) + scalar!(self.gamma))
                * (eval!(c) + (scalar!(self.beta) * xi) + scalar!(self.gamma))
                * commit!(z)
                + (eval!(a) + (scalar!(self.beta) * eval!(sigma1)) + scalar!(self.gamma))
                * (eval!(b) + (scalar!(self.beta) * eval!(sigma2)) + scalar!(self.gamma))
                * (eval!(c) + (scalar!(self.beta) * commit!(sigma3)) + scalar!(self.gamma))
                * eval!(zxi)
              )
            + scalar!(self.alpha) * scalar!(self.alpha) * l1_xi * (commit!(z) + scalar!(neg_one))
            + zh_xi * (commit!(tl) + xi_n * commit!(tm) + xi_2n * commit!(th))
            + scalar!(self.v) * (
                commit!(a) + scalar!(self.v) * (
                    commit!(b) + scalar!(self.v) * (
                        commit!(c) + scalar!(self.v) * (
                            commit!(sigma1) + scalar!(self.v) * commit!(sigma2)
                        )
                    )
                )
            )
            + scalar!(self.v) * (
                eval!(a) + scalar!(self.v) * (
                    eval!(b) + scalar!(self.v) * (
                        eval!(c) + scalar!(self.v) * (
                            eval!(sigma1) + scalar!(self.v) * eval!(sigma2)
                        )
                    )
                )
            );
        Ok(EvaluationProof {s:r, point:self.xi.clone(), w: self.commits.w_z.clone()})
    }

    fn get_proof_wxi (
        &self,
        ctx: &mut C,
    ) -> Result<EvaluationProof<S,P>, Error> {
        let sgate = self.sgate;
        let zxi = CommitQuery::<S, P> {c: Some(self.commits.z), v: Some(self.evals.z_xiw)};
        let s = commit!(zxi) + eval!(zxi);
        let point = {
            let xi = self.xi;
            let w = self.common.w;
            arith_in_ctx!([sgate, ctx] w * xi)?
        };
        Ok(EvaluationProof {s, point, w: self.commits.w_zw.clone()})
    }
}

impl<'a, C:Clone, S:Clone, P:Clone,
    Error:Debug,
    SGate: ContextGroup<C, S, S, Error> + ContextRing<C, S, S, Error>,
    PGate:ContextGroup<C, S, P, Error>>
    SchemaGenerator<'a, S, P> for
    PlonkVerifierParams<'a, C, S, P, Error, SGate, PGate> {
    fn getPointSchemas(&self) -> Vec<EvaluationProof<'a, S, P>> {
      vec![]
    }
}
/*


impl<C: CurveAffine> PlonkVerifierParams<'_, S, P> {

    fn get_e1(
        &mut self,
        cgate: &MainGate<C::ScalarExt>,
        region: &mut Region<'_, C::ScalarExt>,
        offset: &mut usize,
    ) -> Result<S, Error> {
        let r0_xi = self.get_r0(cgate, region, offset)?;
        let v0 = SPE(
            [
                self.evals.sigma2_xi,
                self.evals.sigma1_xi,
                self.evals.c_xi,
                self.evals.b_xi,
                self.evals.a_xi,
                &r0_xi,
            ]
            .to_vec(),
            self.v,
        );
        let v1 = SPE([self.evals.z_xiw].to_vec(), self.one);
        MPE([&v0 as &dyn EvalAggregator<C>, &v1].to_vec(), self.u).aggregate(cgate, region, self.one, offset)
    }

    fn get_f1(
        &mut self,
        cgate: &MainGate<C::ScalarExt>,
        ecc_gate: &BaseFieldEccChip<C>,
        region: &mut Region<'_, C::ScalarExt>,
        offset: &mut usize,
    ) -> Result<AssignedPoint<C::ScalarExt>, Error> {
        let r1 = self.get_r1(cgate, ecc_gate, region, offset)?;
        let v0 = SPC(
            [self.params.sigma2, self.params.sigma1, self.commits.c, self.commits.b, self.commits.a, &r1].to_vec(),
            self.v,
        );
        let v1 = SPC([self.commits.z].to_vec(), self.one);
        MPC([&v0 as &dyn MSMAggregator<C>, &v1].to_vec(), self.u).aggregate(ecc_gate, region, self.one, offset)
    }

    fn get_wx(
        &mut self,
        cgate: &MainGate<C::ScalarExt>,
        ecc_gate: &BaseFieldEccChip<C>,
        ws: Vec<SingleOpeningProof<C>>,
        region: &mut Region<'_, C::ScalarExt>,
        offset: &mut usize,
    ) -> Result<MultiOpeningProof<C>, Error> {
        let e1 = self.get_e1(cgate, region, offset)?;
        let f1 = self.get_f1(cgate, ecc_gate, region, offset)?;
        let mut wxs = Vec::new();
        ws.iter().for_each(|w| {
            wxs.push(w.w.clone());
        });
        let wxs = SPC(wxs.iter().collect(), self.u).aggregate(ecc_gate, region, self.one, offset)?;
        Ok(MultiOpeningProof{w_x: wxs.clone(), w_g: wxs, e: e1, f :f1})

    }
}
*/
