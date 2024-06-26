# MultipartFormData

Parse and create Spec compliant multipart/form-data.

99% of the time you don't need to use this library. Use the `FormData` built into the browser instead. This library is for when you need to parse or create `multipart/form-data` that contains more metadata in the `Content-disposition` header than just the `name` and `filename` attribute.

## Usage

### Create

```typescript
import MultipartFormData from "multipart-form-data";
const formData = new MultipartFormData();
formData.append("file", new Blob(["hello world"]), {
  filename: "hello.txt",
  contentType: "text/plain",
  custom: "metadata",
});
// this is the encoded form data
const data = await formData.blob();
```

### Parse

```typescript
import MultipartFormData from "multipart-form-data";
const data = await response.body.arrayBuffer();
const contentType = response.headers.get("Content-Type");
// parse in the main thread
const formData = MultipartFormData.parse(data, contentType);
// or parse in a worker
const formData = await MultipartFormData.parseAsync(data, contentType);
for (const [name, value, metadata] of formData) {
  console.log(name, value, metadata);
}
```
