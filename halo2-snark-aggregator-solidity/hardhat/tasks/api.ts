const { task } = require("hardhat/config");
const fs = require('fs');
const ethers = require('ethers');
const { exit } = require("process");

const overrides = { gasLimit: 8000000 };

const emtpyHash = "0x0000000000000000000000000000000000000000000000000000000000000000";

async function main(hash, step, file) {
    const provider = new ethers.providers.JsonRpcProvider("http://localhost:8545");
    var res = await provider.send("debug_generateProofForTest", [hash, 0, 0, parseInt(step)]);
    // var res = await provider.send("testIn");
    // var res = await provider.send("ku_blockNumber");
    console.log(res)
    fs.writeFileSync(file, JSON.stringify(res));
    console.log("wrote proof to " + file);
}

task("generateOsp", "generate osp")
    .addParam("hash", "the transaction hash to prove")
    .addParam("step", "the step to prove")
    .addParam("file", "where to save the proof")
    .setAction(async (taskArgs) => {
        await main(taskArgs.hash, taskArgs.step, taskArgs.file);
    });

module.exports = {};
// 0xc68b6051c68bb583ac7f0ce3561300730962d5802fd5c0c9b2ab9329f7b8876f