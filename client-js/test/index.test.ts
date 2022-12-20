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
import SET_TOKEN_METADATA_DEPLOY_ARGS_JSON from "./jsons/set-metadata-args.json";
import APPROVE_DEPLOY_ARGS_JSON from "./jsons/approve-args.json";
import APPROVE_ALL_DEPLOY_ARGS_JSON from "./jsons/approve-all-args.json";
import BALANCE_OF_DEPLOY_ARGS_JSON from "./jsons/balance-of-args.json";
import GET_APPROVED_DEPLOY_ARGS_JSON from "./jsons/get-approved-args.json";
import OWNER_OF_DEPLOY_ARGS_JSON from "./jsons/owner-of-args.json";
import REGISTER_ARGS_JSON from "./jsons/register-args.json";
import UPDATED_RECEPIENT_ARGS_JSON from "./jsons/updated-reciepients-args.json";
import MIGRATE_ARGS_JSON from "./jsons/migrate-args.json";

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

    expect(installDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      INSTALL_ARGS_JSON
    );
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

    expect(setVariablesDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.StoredContractByHash.entry_point).toEqual(
      "set_variables"
    );
    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      SET_VARIABLES_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'register'", async () => {
    const registerDeploy = await cc.register(
      {
        tokenOwner: MOCKED_OWNER_PUBKEY,
      },
      "250000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(registerDeploy) as any;

    expect(registerDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.StoredContractByHash.entry_point).toEqual(
      "register_owner"
    );
    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      REGISTER_ARGS_JSON
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
      { useSessionCode: true },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(mintDeploy) as any;

    expect(mintDeploy).toBeInstanceOf(DeployUtil.Deploy);
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

    expect(burnDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.StoredContractByHash.entry_point).toEqual(
      "burn"
    );
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
      { useSessionCode: true },
      "13000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(transferDeploy) as any;

    expect(transferDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      TRANSFER_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'set_token_metadata'", async () => {
    const setTokenMetaDataDeploy = await cc.setTokenMetadata(
      { tokenMetaData: { color: "Red", size: "XLarge", material: "Cotton" } },
      "13000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(setTokenMetaDataDeploy) as any;

    expect(setTokenMetaDataDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.StoredContractByHash.entry_point).toEqual(
      "set_token_metadata"
    );
    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      SET_TOKEN_METADATA_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'approve'", async () => {
    const setApproveDeploy = await cc.approve(
      {
        operator: MOCKED_RECIPIENT_PUBKEY,
        tokenId: "0",
      },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(setApproveDeploy) as any;

    expect(setApproveDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.StoredContractByHash.entry_point).toEqual(
      "approve"
    );
    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      APPROVE_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'set_approval_for_all'", async () => {
    const setApproveAllDeploy = await cc.approveAll(
      {
        tokenOwner: MOCKED_OWNER_PUBKEY,
        operator: MOCKED_RECIPIENT_PUBKEY,
        approveAll: true,
      },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(setApproveAllDeploy) as any;

    expect(setApproveAllDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.StoredContractByHash.entry_point).toEqual(
      "set_approval_for_all"
    );
    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      APPROVE_ALL_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'balance_of'", async () => {
    const balanceOfDeploy = await cc.storeBalanceOf(
      {
        tokenOwner: MOCKED_OWNER_PUBKEY,
        keyName: "abc",
      },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(balanceOfDeploy) as any;

    expect(balanceOfDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      BALANCE_OF_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'get_approved'", async () => {
    const getApprovedDeploy = await cc.storeGetApproved(
      {
        keyName: "def",
      },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(getApprovedDeploy) as any;

    expect(getApprovedDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      GET_APPROVED_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'updated_reciepients'", async () => {
    const updatedReceiptsDeploy = await cc.updatedReceipts(
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(updatedReceiptsDeploy) as any;

    expect(updatedReceiptsDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      UPDATED_RECEPIENT_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'owner_of'", async () => {
    const ownerOfDeploy = await cc.storeOwnerOf(
      {
        keyName: "def",
        tokenId: "0",
      },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(ownerOfDeploy) as any;

    expect(ownerOfDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.ModuleBytes.args).toEqual(
      OWNER_OF_DEPLOY_ARGS_JSON
    );
  });

  it("Should correctly construct deploy for 'migrate'", async () => {
    const ownerOfDeploy = await cc.migrate(
      { collectionName: "my-collection" },
      "1000000000",
      keyPair.publicKey
    );

    const JSONDeploy = DeployUtil.deployToJson(ownerOfDeploy) as any;

    expect(ownerOfDeploy).toBeInstanceOf(DeployUtil.Deploy);
    expect(JSONDeploy.deploy.session.StoredContractByHash.args).toEqual(
      MIGRATE_ARGS_JSON
    );
  });
});
