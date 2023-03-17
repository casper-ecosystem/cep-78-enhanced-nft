import {
  CLValueParsers,
  CLMap,
  CLString,
  CLValueBuilder,
  CLTypeTag,
  CLValue,
  CasperServiceByJsonRPC,
} from "casper-js-sdk";

import { Parser } from "@make-software/ces-js-parser";

import { CEP78_CONTRACT_PACKAGE } from "./constants";
import { EventItem, EventParsed, CEP47Events, Transform } from "./types";

export const CEP47EventParserFactory =
  ({
    contractPackageHash,
    eventNames,
  }: {
    contractPackageHash: string;
    eventNames: CEP47Events[];
  }) =>
  (value: EventItem) => {
    if (!value.body.DeployProcessed.execution_result.Success) {
      return null;
    }

    if (value.body.DeployProcessed.execution_result.Success) {
      const { transforms } =
        value.body.DeployProcessed.execution_result.Success.effect;

      const cep47Events = transforms.reduce(
        (acc: EventParsed[], val: Transform) => {
          if (val.transform.WriteCLValue?.cl_type === "Any") {
            const maybeCLValue = CLValueParsers.fromBytesWithType(
              Buffer.from(val.transform.WriteCLValue?.bytes, "hex")
            );
            const clValue = maybeCLValue.unwrap();

            if (clValue?.clType().tag === CLTypeTag.Map) {
              const hash = (clValue as CLMap<CLValue, CLValue>).get(
                CLValueBuilder.string(CEP78_CONTRACT_PACKAGE)
              );

              const hashToCompare = (hash as CLString).value().slice(21);

              const event = (clValue as CLMap<CLValue, CLValue>)
                .get(CLValueBuilder.string("event_type"))
                .value() as string;

              if (
                hash &&
                hashToCompare === contractPackageHash.slice(5) &&
                event &&
                eventNames.includes(event as CEP47Events)
              ) {
                /* eslint-disable-next-line no-param-reassign */
                acc = [...acc, { name: event, clValue }];
              }
            }
          }
          return acc;
        },
        []
      );

      // For now we're returning error: null because failed deploys doesn't contain contract-package-hash so we can't identify them.
      // But as this is part of node-1.5 release I'm keeping this so we won't change interface here.
      return { error: null, success: !!cep47Events.length, data: cep47Events };
    }

    return null;
  };

export const CESEventParserFactory =
  ({
    contractHashes,
    // TODO: IDEALLY in future I would love to have here a schema as an argument instead of casperClient. That way the whole thing can be initialized offline as the whole client.
    casperClient,
  }: {
    contractHashes: string[];
    casperClient: CasperServiceByJsonRPC;
  }) =>
  async (event: EventItem) => {
    const validatedHashes = contractHashes.map((hash) =>
      hash.startsWith("hash-") ? hash.slice(5) : hash
    );
    const parser = await Parser.create(casperClient, validatedHashes);

    try {
      const toParse = event.body.DeployProcessed.execution_result;
      const events = parser.parseExecutionResult(toParse);
      return { error: null, success: !!events.length, data: events };
    } catch (error: unknown) {
      return { error, success: false, data: null };
    }
  };
