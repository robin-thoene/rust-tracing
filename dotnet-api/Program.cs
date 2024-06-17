using System.Diagnostics;
using OpenTelemetry;
using OpenTelemetry.Exporter;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using Serilog;
using Serilog.Events;

const string OTEL_TRACEID_RESP_HEADER_NAME = "trace_id";
var builder = WebApplication.CreateBuilder(args);
// Setup open telemetry.
const string serviceName = "dotnet-api";
const string grpcEndpoint = "http://localhost:4317";
builder.Services.AddOpenTelemetry()
    .UseOtlpExporter(OtlpExportProtocol.Grpc, new Uri(grpcEndpoint))
    .ConfigureResource(resource => resource.AddService(serviceName))
    .WithTracing(tracing => tracing.AddAspNetCoreInstrumentation().AddHttpClientInstrumentation());
builder.Host.UseSerilog((ctx, services, config) =>
    {
        config.WriteTo.Console();
        config.WriteTo.OpenTelemetry(opt =>
        {
            opt.Endpoint = grpcEndpoint;
            opt.ResourceAttributes.Add("service.name", serviceName);
            opt.Protocol = Serilog.Sinks.OpenTelemetry.OtlpProtocol.Grpc;
            opt.RestrictedToMinimumLevel = LogEventLevel.Warning;
        });
    });
// Add services to the container.
// Learn more about configuring Swagger/OpenAPI at https://aka.ms/aspnetcore/swashbuckle
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();

var app = builder.Build();

// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.UseSwagger();
    app.UseSwaggerUI();
}

var summaries = new[]
{
    "Freezing", "Bracing", "Chilly", "Cool", "Mild", "Warm", "Balmy", "Hot", "Sweltering", "Scorching"
};

app.MapGet("/weatherforecast", (ILogger<Program> logger, HttpContext httpCtx) =>
{
    var traceId = Activity.Current?.RootId;
    httpCtx.Response.Headers.Append(OTEL_TRACEID_RESP_HEADER_NAME, traceId);
    var forecast = Enumerable.Range(1, 5).Select(index =>
        new WeatherForecast
        (
            DateOnly.FromDateTime(DateTime.Now.AddDays(index)),
            Random.Shared.Next(-20, 55),
            summaries[Random.Shared.Next(summaries.Length)]
        ))
        .ToArray();
    logger.LogError("Testing logging ... {@foreacast}", forecast);

    var random = new Random();
    var randomBoolean = random.Next(2) == 0;
    if (randomBoolean)
    {
        // Simulate an error.
        return Results.Problem();
    }
    else
    {
        // Return normal response.
        return Results.Ok(forecast);
    }
});
app.MapGet("/downstream-api-status", async (HttpContext httpCtx) =>
{
    var traceId = Activity.Current?.RootId;
    httpCtx.Response.Headers.Append(OTEL_TRACEID_RESP_HEADER_NAME, traceId);
    using var httpClient = new HttpClient();
    httpClient.BaseAddress = new Uri("http://localhost:9000");
    httpClient.DefaultRequestHeaders.Add("user-agent", "dotnet-http-client");
    var response = await httpClient.GetAsync("status");
    var result = await response.Content.ReadAsStringAsync();
    return result;
});

app.Run();

record WeatherForecast(DateOnly Date, int TemperatureC, string? Summary)
{
    public int TemperatureF => 32 + (int)(TemperatureC / 0.5556);
}
