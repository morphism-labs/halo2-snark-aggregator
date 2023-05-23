import fs from "fs";
import { Signer, Contract, BigNumber } from 'ethers'
import chai from 'chai'
import hre from 'hardhat'

const { expect } = chai
const { ethers } = require('hardhat')

function bufferToUint256BE(buffer: Buffer) {
  let buffer256:BigNumber[] = [];
  for (let i = 0; i < buffer.length / 32; i++) {
    let v = BigNumber.from(0);
    for (let j = 0; j < 32; j++) {
      v = v.shl(8);
      v = v.add(buffer[i * 32 + j]);
    }
    buffer256.push(v);
  }

  return buffer256;
}

function bufferToUint256LE(buffer: Buffer) {
  let buffer256:BigNumber[] = [];
  for (let i = 0; i < buffer.length / 32; i++) {
    let v = BigNumber.from(0);
    let shft = BigNumber.from(1);
    for (let j = 0; j < 32; j++) {
      v = v.add(shft.mul(buffer[i * 32 + j]));
      shft = shft.mul(256);
    }
    buffer256.push(v);
  }

  return buffer256;
}

describe("Verifier", () => {
  let accounts: Signer[]
  let verifier: Contract

  before(async () => {
    accounts = await ethers.getSigners()
    await deployVerifier()
  });

  const deployVerifier = async () => {
    const VerifierFactory = await hre.ethers.getContractFactory('Verifier',accounts[0])
    verifier = await VerifierFactory.deploy()
  }

  it('Assigns initial balance', async () => {
    let proof = fs.readFileSync(
      "output/verify_circuit_proof.data"
    );
    let final_pair = fs.readFileSync(
      "output/verify_circuit_final_pair.data"
  //	  "output/verify_circuit_instance.data"
    );
    console.log("proof length", proof.length);
  
    console.log(bufferToUint256LE(final_pair));  
    let a = await verifier.verify(
      bufferToUint256LE(proof),
      bufferToUint256LE(final_pair),
    );
    console.log(a.toString());
  })
});
