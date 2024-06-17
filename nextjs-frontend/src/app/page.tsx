export default async function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div>
        <div>
          Check the <a href="/weatherforecast">weatherforecast</a>
        </div>
      </div>
      <div>
        <div>
          Check the <a href="/downstream-status">axum downstram api status</a>{" "}
          directly
        </div>
      </div>
      <div>
        <div>
          Check the{" "}
          <a href="/axum/downstream-status">
            axum downstram api status behind axum
          </a>{" "}
          proxy
        </div>
      </div>
      <div>
        <div>
          Check the{" "}
          <a href="/dotnet/downstream-status">
            axum downstram api status behind dotnet
          </a>{" "}
          proxy
        </div>
      </div>
      <div>
        <div>Get a greeting</div>
        TODO
      </div>
    </main>
  );
}
