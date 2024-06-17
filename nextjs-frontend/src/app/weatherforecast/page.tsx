export default async function Weather() {
  const response = await fetch("http://localhost:5240/weatherforecast", {
    cache: "no-store",
  });
  const traceId = response.headers.get("trace_id");
  console.log(traceId);
  let dataToDisplay;
  if (response.status == 200) {
    dataToDisplay = JSON.stringify(await response.json());
  } else {
    dataToDisplay = "An error occured!";
  }
  return (
    <div className="flex flex-col">
      <div>{dataToDisplay}</div>
      <div className="mt-6">Check out the traces using this id</div>
      <div className="font-bold">{traceId}</div>
    </div>
  );
}
