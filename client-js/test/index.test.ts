import {
  CEP78Client,
  NFTOwnershipMode,
  NFTKind,
  NFTMetadataKind,
  NFTIdentifierMode,
  MetadataMutability,
  MintingMode,
} from "../src/index";

import {
  Keys,
  DeployUtil,
  CLValue,
  CLPublicKey,
  CLKey,
  Contracts,
} from "casper-js-sdk";

import INSTALL_ARGS_JSON from "./jsons/install-args.json";
import SET_VARIABLES_ARGS_JSON from "./jsons/set-variables-args.json";
import MINT_DEPLOY_ARGS_JSON from "./jsons/mint-args.json";
import BURN_DEPLOY_ARGS_JSON from "./jsons/burn-args.json";
import TRANSFER_DEPLOY_ARGS_JSON from "./jsons/transfer-args.json";


describe("CEP78Client", () => {
  const MOCKED_OWNER_PUBKEY = CLPublicKey.fromHex(
    "0145fb72c75e1b459839555d70356a5e6172e706efa204d86c86050e2f7878960f"
  );
  const MOCKED_RECIPIENT_PUBKEY = CLPublicKey.fromHex(
    "0112b28459a5c90b7c90f700788302d463b5c29acfef1dd3da5d1ef162f71061f7"
  );

  const keyPair = Keys.Ed25519.new();
  const cc = new CEP78Client("http://localhost:11101/rpc", "casper-net-1");

  it("Should correctly construct contract install deploy", async () => {
    const installDeploy = await cc.install(
      {
        collectionName: "my-collection",
        collectionSymbol: "MY-NFTS",
        totalTokenSupply: "1000",
        ownershipMode: NFTOwnershipMode.Transferable,
        nftKind: NFTKind.Physical,
        jsonSchema: {
          properties: {
            color: { name: "color", description: "", required: true },
            size: { name: "size", description: "", required: true },
            material: { name: "material", description: "", required: true },
            condition: { name: "condition", description: "", required: false },
          },
        },
        nftMetadataKind: NFTMetadataKind.CustomValidated,
        identifierMode: NFTIdentifierMode.Ordinal,
        metadataMutability: MetadataMutability.Immutable,
        mintingMode: MintingMode.Installer,
      },
      "250000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(installDeploy) as any;

    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      INSTALL_ARGS_JSON
    );
  });

  it("Should correctly initialize inself when correct hash is provided", async () => {
    await cc.setContractHash(
      "hash-0c0f9056626a55273bd8238f595908f2e4d78acc2546bf1f78f39f814bc60fe4"
    );

    expect(cc.contractClient).toBeInstanceOf(Contracts.Contract);
    expect(cc.contractHashKey).toBeInstanceOf(CLKey);
  });

  it("Should correctly construct deploy for 'set_variables'", async () => {
    const setVariablesDeploy = await cc.setVariables(
      {
        allowMinting: true,
      },
      "250000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(setVariablesDeploy) as any;

    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      SET_VARIABLES_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'mint'", async () => {
    const mintDeploy = await cc.mint(
      {
        owner: MOCKED_OWNER_PUBKEY,
        meta: {
          color: "Blue",
          size: "Medium",
          material: "Aluminum",
          condition: "Used",
        },
      },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(mintDeploy) as any;

    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      MINT_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'burn'", async () => {
    const burnDeploy = await cc.burn(
      { tokenId: "0" },
      "13000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(burnDeploy) as any;

    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      BURN_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'transfer'", async () => {
    const transferDeploy = await cc.transfer(
      {
        tokenId: "0",
        source: MOCKED_OWNER_PUBKEY,
        target: MOCKED_RECIPIENT_PUBKEY,
      },
      "13000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(transferDeploy) as any;

    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      TRANSFER_DEPLOY_ARGS_JSON
    );
  });
  // const transferDeploy = await cc.transfer(
  //   {
  //     tokenId: "0",
  //     source: FAUCET_KEYS.publicKey,
  //     target: USER1_KEYS.publicKey,
  //   },
  //   "13000000000",
  //   FAUCET_KEYS.publicKey,
  //   [FAUCET_KEYS]
  // );
});
