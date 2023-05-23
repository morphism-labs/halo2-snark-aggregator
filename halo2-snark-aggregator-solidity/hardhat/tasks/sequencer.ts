import "@nomiclabs/hardhat-web3";
import "@nomiclabs/hardhat-ethers";
import "@nomiclabs/hardhat-waffle";

import { task, types } from "hardhat/config";
import { ethers } from "ethers";

const l1RpcProvider = new ethers.providers.JsonRpcProvider('http://127.0.0.1:9545')
const amount = ethers.utils.parseEther('100')
const deposit = ethers.utils.parseEther('1')
const prvKeys = [
  '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
  '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d',
  '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a',
  '0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6',
  '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a',
  '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba'
]
const nodeIDs = [
  '0x7a6b7d46da49c9c5045a6ecb387415d7827506f1aff9bead32240772a39544d4',
  '0x93c6d59747ef03a1927f9d190a0038a4b64bdd77b7383b29244acce60c2ef23e',
  '0x2a1776580e7155022477c406dbd091983712d222d216d5e11d3c99b4e47671a8',
  '0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6',
  // '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a',
  // '0x42e5b20fac9b9f9087f679229e8be68f10c4293f6e7cf9366dba528c767d82cd',
  // '0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba'
]
var wallets = new Array();

for (var i = 0; i < prvKeys.length; i++) {
  wallets.push(new ethers.Wallet(prvKeys[i], l1RpcProvider))
}

task("deployToken")
    .setAction(async (taskArgs, hre) => {
        const tokenFactory = await hre.ethers.getContractFactory('BitTokenERC20')
        const token = await tokenFactory.deploy('name', 'symbol')
        const res = await token.deployed();
        console.log("export BIT_ERC20_TOKEN=%s \n TX_HASH: %s", token.address.toLocaleLowerCase(),token.deployTransaction.hash);
    });

task("deploy")
  .setAction(async (taskArgs, hre) => {
    const tokenFactory = await hre.ethers.getContractFactory('BitTokenERC20')
    const token = await tokenFactory.deploy('name', 'symbol')
    await token.deployed();
    console.log("export BIT_ERC20_TOKEN=%s", token.address.toLocaleLowerCase());

    for (var i = 0; i < wallets.length; i++) {
      await token.connect(wallets[i]).mint(amount)
      // console.log("balance of", wallets[i].address, "is:", await token.balanceOf(wallets[i].address))
    }

    const sequencerFactory = await hre.ethers.getContractFactory('Sequencer')
    const sequencer = await sequencerFactory.deploy()
    await sequencer.deployed()
    await sequencer.initialize(token.address)
    console.log("export ENV_SEQUENCER_CONTRACT_ADDRESS=%s", sequencer.address.toLocaleLowerCase());
    console.log("sequencer contract owner :", await sequencer.owner())
    console.log("sequencer bit address :", await sequencer.bitToken())

    // create sequencer
    for (var i = 1; i < nodeIDs.length; i++) {
      await token.connect(wallets[i]).approve(sequencer.address, deposit)
      await sequencer.connect(wallets[i]).createSequencer(deposit, wallets[i].address, nodeIDs[i])
    }
    await sequencer.updateSequencerLimit(100)

    await sequencer.updateScheduler(wallets[0].address)
    console.log("scheduler is :", await sequencer.scheduler())
  });

task("updateScheduler")
  .addParam("sequencer")
  .addParam("scheduler")
  .setAction(async (taskArgs, hre) => {
    const sequencerFactory = await hre.ethers.getContractFactory('Sequencer')
    const sequencer = sequencerFactory.attach(taskArgs.sequencer)
    await sequencer.updateScheduler(taskArgs.scheduler)
    console.log(await sequencer.scheduler())
  });


task("init")
  // .addParam("token")
  .setAction(async (taskArgs, hre) => {
    const tokenFactory = await hre.ethers.getContractFactory('BitTokenERC20')
    const token = tokenFactory.attach("0x5FbDB2315678afecb367f032d93F642f64180aa3")
    for (var i = 0; i < wallets.length; i++) {

      var res = await token.connect(wallets[i]).mint(amount)
      console.log("res", res, "address:", wallets[i].addres)
      // console.log("balance of", wallets[i].address, "is:", await token.balanceOf(wallets[i].address))
    }
  });

task("createSequencer")
  .setAction(async (taskArgs, hre) => {
    const sequencerFactory = await hre.ethers.getContractFactory('Sequencer')
    const sequencer = sequencerFactory.attach("0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0")
    // const sequencer = sequencerFactory.attach("0x36fCf02Fc651c0b7ef2ECA446Dd2405364F85337")

    const tokenFactory = await hre.ethers.getContractFactory('BitTokenERC20')
    const token = tokenFactory.attach("0x5FbDB2315678afecb367f032d93F642f64180aa3")
    // const token = tokenFactory.attach("0x92aBAD50368175785e4270ca9eFd169c949C4ce1")

    // // create sequencer
    const useIndex = 3
    await token.connect(wallets[useIndex]).mint(amount)
    console.log("balances: ",await token.balanceOf(wallets[useIndex].address))
    await token.connect(wallets[useIndex]).approve(sequencer.address, deposit)
    console.log("allowance: ",await token.allowance(wallets[useIndex].address,sequencer.address))
    await sequencer.connect(wallets[useIndex]).createSequencer(deposit, wallets[useIndex].address, nodeIDs[useIndex])
  });

task("withdrawAll")
 .setAction(async (taskArgs, hre) => {
    const sequencerFactory = await hre.ethers.getContractFactory('Sequencer')
    const sequencer = sequencerFactory.attach("0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0")
    // const sequencer = sequencerFactory.attach("0x36fCf02Fc651c0b7ef2ECA446Dd2405364F85337")

    // 0xBe0F340075060F856612d91e17cAe599dE92C745
    // // create sequencer
    const useIndex = 3
    console.log(wallets[useIndex].address)
    await sequencer.connect(wallets[useIndex]).withdrawAll()
  });

task("deposit")
 .setAction(async (taskArgs, hre) => {
    const sequencerFactory = await hre.ethers.getContractFactory('Sequencer')
    const sequencer = sequencerFactory.attach("0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0")
    const tokenFactory = await hre.ethers.getContractFactory('BitTokenERC20')
    const token = tokenFactory.attach("0x5FbDB2315678afecb367f032d93F642f64180aa3")
    
    // 0xBe0F340075060F856612d91e17cAe599dE92C745
    // // create sequencer
    const useIndex = 2
    await token.connect(wallets[useIndex]).mint(deposit)
    await token.connect(wallets[useIndex]).approve(sequencer.address, deposit)
    await sequencer.connect(wallets[useIndex]).deposit(deposit)
  });