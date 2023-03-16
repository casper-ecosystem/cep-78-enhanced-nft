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
    if (value.body.DeployProcessed.execution_result.Success) {
      const { transforms } =
        value.body.DeployProcessed.execution_result.Success.effect;

      const cep47Events = transforms.reduce(
        (acc: EventParsed[], val: Transform) => {
          if (
            val.transform.WriteCLValue &&
            val.transform.WriteCLValue.cl_type === "Any"
          ) {
            const maybeCLValue = CLValueParsers.fromBytesWithType(
              Buffer.from(val.transform.WriteCLValue?.bytes, "hex")
            );
            const clValue = maybeCLValue.unwrap();

            if (clValue && clValue.clType().tag === CLTypeTag.Map) {
              const hash = (clValue as CLMap<CLValue, CLValue>).get(
                CLValueBuilder.string("cep78_contract_package")
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
    const parser = await Parser.create(casperClient, contractHashes.map(c => c.slice(5)));

    try {
      const toParse = event.body.DeployProcessed.execution_result;
      const events = parser.parseExecutionResult(toParse);
      return { error: null, success: !!events.length, data: events };
    } catch (error: unknown) {
      return { error, success: false, data: null };
    }

  };
