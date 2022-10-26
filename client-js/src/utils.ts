import { CLValueParsers } from "casper-js-sdk";

const isObject = (item: unknown) => typeof item === "object";

// TODO: Add typings
export const getMintedId = (transforms: any) => {
  const addKeysTransform = transforms.find(
    (t: any) => isObject(t.transform) && t.transform.AddKeys
  );

  if (addKeysTransform) {
    const lookupDict = addKeysTransform.transform.AddKeys[0].key;
    const lookupTransform = transforms.find(
      (t: any) =>
        t.key === lookupDict &&
        isObject(t.transform) &&
        t.transform.WriteCLValue
    );

    const buff = Buffer.from(
      lookupTransform.transform.WriteCLValue.bytes,
      "hex"
    );

    const data = CLValueParsers.fromBytesWithType(buff).unwrap().toJSON();

    return data[data.length - 1];
  }

  return null;
};
