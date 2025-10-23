#!/usr/bin/env python3
"""
Layer5 Performance Benchmarking Script

This script runs comprehensive performance benchmarks for Layer5 components
and generates optimization recommendations.
"""

import asyncio
import json
import time
import argparse
import statistics
from typing import Dict, List, Any
import aiohttp
import numpy as np
from dataclasses import dataclass, asdict
from datetime import datetime

@dataclass
class BenchmarkResult:
    """Container for benchmark results"""
    timestamp: str
    component: str
    metric: str
    value: float
    unit: str
    target: float
    status: str  # "PASS", "WARN", "FAIL"

@dataclass
class BenchmarkConfig:
    """Configuration for benchmarks"""
    layer5_url: str = "http://localhost:8080"
    duration: int = 300  # 5 minutes
    concurrency: int = 10
    kpi_batch_size: int = 100
    num_agents: int = 1000

class Layer5Benchmarker:
    """Main benchmarking class"""

    def __init__(self, config: BenchmarkConfig):
        self.config = config
        self.results: List[BenchmarkResult] = []
        self.session = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def health_check(self) -> bool:
        """Check if Layer5 is healthy"""
        try:
            async with self.session.get(f"{self.config.layer5_url}/health") as resp:
                return resp.status == 200
        except Exception as e:
            print(f"Health check failed: {e}")
            return False

    async def benchmark_kpi_ingestion(self) -> List[BenchmarkResult]:
        """Benchmark KPI ingestion performance"""
        print("Benchmarking KPI ingestion...")

        results = []
        start_time = time.time()

        # Generate test KPI data
        test_kpis = self.generate_test_kpis()

        # Send KPIs concurrently
        tasks = []
        for i in range(0, len(test_kpis), self.config.kpi_batch_size):
            batch = test_kpis[i:i + self.config.kpi_batch_size]
            tasks.append(self.send_kpi_batch(batch))

        # Execute and measure
        batch_times = await asyncio.gather(*tasks)

        total_time = time.time() - start_time
        throughput = len(test_kpis) / total_time
        avg_latency = statistics.mean(batch_times)
        p95_latency = np.percentile(batch_times, 95)

        # Record results
        results.append(BenchmarkResult(
            timestamp=datetime.now().isoformat(),
            component="kpi_ingestion",
            metric="throughput",
            value=throughput,
            unit="kpi/s",
            target=1000,
            status="PASS" if throughput >= 1000 else "FAIL"
        ))

        results.append(BenchmarkResult(
            timestamp=datetime.now().isoformat(),
            component="kpi_ingestion",
            metric="avg_latency",
            value=avg_latency * 1000,
            unit="ms",
            target=50,
            status="PASS" if avg_latency * 1000 <= 50 else "FAIL"
        ))

        results.append(BenchmarkResult(
            timestamp=datetime.now().isoformat(),
            component="kpi_ingestion",
            metric="p95_latency",
            value=p95_latency * 1000,
            unit="ms",
            target=100,
            status="PASS" if p95_latency * 1000 <= 100 else "WARN"
        ))

        return results

    async def send_kpi_batch(self, kpis: List[Dict]) -> float:
        """Send a batch of KPIs and measure response time"""
        start_time = time.time()
        try:
            async with self.session.post(
                f"{self.config.layer5_url}/kpi",
                json=kpis,
                headers={"Content-Type": "application/json"}
            ) as resp:
                if resp.status != 200:
                    print(f"Error sending KPI batch: {resp.status}")
        except Exception as e:
            print(f"Exception sending KPI batch: {e}")

        return time.time() - start_time

    def generate_test_kpis(self) -> List[Dict]:
        """Generate test KPI data"""
        kpis = []
        base_time = datetime.now().timestamp()

        for agent_id in range(self.config.num_agents):
            for i in range(10):  # 10 KPIs per agent
                kpi = {
                    "timestamp": base_time + i,
                    "agent_id": f"agent_{agent_id}",
                    "task_id": f"task_{agent_id}_{i}",
                    "metrics": {
                        "cpu_usage": np.random.normal(0.7, 0.2),
                        "memory_usage": np.random.normal(0.6, 0.15),
                        "response_time": np.random.normal(100, 20),
                        "throughput": np.random.normal(1000, 100),
                        "error_rate": np.random.normal(0.01, 0.005)
                    },
                    "metadata": {
                        "version": "1.0",
                        "environment": "test"
                    }
                }
                kpis.append(kpi)

        return kpis

    async def benchmark_optimization(self) -> List[BenchmarkResult]:
        """Benchmark optimization engine performance"""
        print("Benchmarking optimization engine...")

        results = []

        # Test different optimization algorithms
        algorithms = ["bandit", "bayesian", "gradient"]

        for algorithm in algorithms:
            start_time = time.time()

            # Send optimization request
            try:
                async with self.session.post(
                    f"{self.config.layer5_url}/optimize",
                    json={
                        "agent_id": "benchmark_agent",
                        "algorithm": algorithm,
                        "parameters": {
                            "learning_rate": 0.01,
                            "momentum": 0.9,
                            "exploration_rate": 0.1
                        }
                    }
                ) as resp:
                    if resp.status == 200:
                        response_data = await resp.json()
                        optimization_time = time.time() - start_time

                        results.append(BenchmarkResult(
                            timestamp=datetime.now().isoformat(),
                            component="optimization",
                            metric=f"{algorithm}_latency",
                            value=optimization_time * 1000,
                            unit="ms",
                            target=100,
                            status="PASS" if optimization_time * 1000 <= 100 else "FAIL"
                        ))

                        # Check optimization quality
                        if "confidence" in response_data:
                            confidence = response_data["confidence"]
                            results.append(BenchmarkResult(
                                timestamp=datetime.now().isoformat(),
                                component="optimization",
                                metric=f"{algorithm}_confidence",
                                value=confidence,
                                unit="ratio",
                                target=0.9,
                                status="PASS" if confidence >= 0.9 else "WARN"
                            ))
            except Exception as e:
                print(f"Error benchmarking {algorithm}: {e}")

        return results

    async def benchmark_pattern_recognition(self) -> List[BenchmarkResult]:
        """Benchmark pattern recognition performance"""
        print("Benchmarking pattern recognition...")

        results = []

        # Generate time series data with known patterns
        time_series = self.generate_time_series_data()

        start_time = time.time()
        try:
            async with self.session.post(
                f"{self.config.layer5_url}/analyze",
                json={"time_series": time_series}
            ) as resp:
                if resp.status == 200:
                    analysis_time = time.time() - start_time

                    results.append(BenchmarkResult(
                        timestamp=datetime.now().isoformat(),
                        component="pattern_recognition",
                        metric="analysis_latency",
                        value=analysis_time * 1000,
                        unit="ms",
                        target=200,
                        status="PASS" if analysis_time * 1000 <= 200 else "WARN"
                    ))
        except Exception as e:
            print(f"Error in pattern recognition benchmark: {e}")

        return results

    def generate_time_series_data(self) -> List[Dict]:
        """Generate time series data with trends and anomalies"""
        data = []
        base_time = datetime.now().timestamp()

        # Generate 1000 data points with trend and seasonal pattern
        for i in range(1000):
            timestamp = base_time + (i * 60)  # 1-minute intervals

            # Trend component
            trend = 0.001 * i

            # Seasonal component (daily cycle)
            seasonal = 0.3 * np.sin(2 * np.pi * i / 1440)  # 1440 minutes in a day

            # Noise
            noise = np.random.normal(0, 0.1)

            # Anomalies (5% of data points)
            anomaly = 0
            if np.random.random() < 0.05:
                anomaly = np.random.normal(2, 0.5)

            value = trend + seasonal + noise + anomaly

            data.append({
                "timestamp": timestamp,
                "value": max(0, value),  # Ensure non-negative
                "metadata": {"source": "benchmark"}
            })

        return data

    async def run_full_benchmark(self) -> Dict[str, Any]:
        """Run complete benchmark suite"""
        print("Starting Layer5 performance benchmark...")

        if not await self.health_check():
            raise Exception("Layer5 health check failed")

        all_results = []

        # Run individual benchmarks
        all_results.extend(await self.benchmark_kpi_ingestion())
        all_results.extend(await self.benchmark_optimization())
        all_results.extend(await self.benchmark_pattern_recognition())

        # Generate summary
        summary = self.generate_summary(all_results)

        # Save results
        self.save_results(all_results, summary)

        return summary

    def generate_summary(self, results: List[BenchmarkResult]) -> Dict[str, Any]:
        """Generate benchmark summary"""
        summary = {
            "timestamp": datetime.now().isoformat(),
            "total_tests": len(results),
            "passed": len([r for r in results if r.status == "PASS"]),
            "warnings": len([r for r in results if r.status == "WARN"]),
            "failed": len([r for r in results if r.status == "FAIL"]),
            "overall_status": "PASS"
        }

        if summary["failed"] > 0:
            summary["overall_status"] = "FAIL"
        elif summary["warnings"] > 0:
            summary["overall_status"] = "WARN"

        # Component-wise summary
        components = {}
        for result in results:
            if result.component not in components:
                components[result.component] = {
                    "tests": 0,
                    "passed": 0,
                    "warnings": 0,
                    "failed": 0,
                    "metrics": {}
                }

            components[result.component]["tests"] += 1
            components[result.component][result.status.lower()] += 1
            components[result.component]["metrics"][result.metric] = {
                "value": result.value,
                "unit": result.unit,
                "target": result.target,
                "status": result.status
            }

        summary["components"] = components

        return summary

    def save_results(self, results: List[BenchmarkResult], summary: Dict[str, Any]):
        """Save benchmark results to files"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        # Save detailed results
        with open(f"layer5_benchmark_{timestamp}.json", "w") as f:
            json.dump({
                "summary": summary,
                "results": [asdict(r) for r in results]
            }, f, indent=2)

        # Save summary report
        with open(f"layer5_benchmark_summary_{timestamp}.txt", "w") as f:
            f.write("Layer5 Performance Benchmark Summary\n")
            f.write("=" * 50 + "\n\n")
            f.write(f"Timestamp: {summary['timestamp']}\n")
            f.write(f"Overall Status: {summary['overall_status']}\n")
            f.write(f"Tests: {summary['total_tests']} | Passed: {summary['passed']} | Warnings: {summary['warnings']} | Failed: {summary['failed']}\n\n")

            for component, data in summary['components'].items():
                f.write(f"{component.upper()}:\n")
                f.write(f"  Tests: {data['tests']}\n")
                f.write(f"  Status: {'PASS' if data['failed'] == 0 else 'FAIL' if data['failed'] > 0 else 'WARN'}\n")
                for metric, info in data['metrics'].items():
                    f.write(f"  {metric}: {info['value']}{info['unit']} (target: {info['target']}{info['unit']}) [{info['status']}]\n")
                f.write("\n")

        print(f"Benchmark results saved to layer5_benchmark_{timestamp}.json")
        print(f"Summary report saved to layer5_benchmark_summary_{timestamp}.txt")

async def main():
    """Main benchmark execution"""
    parser = argparse.ArgumentParser(description="Layer5 Performance Benchmark")
    parser.add_argument("--url", default="http://localhost:8080", help="Layer5 service URL")
    parser.add_argument("--duration", type=int, default=300, help="Benchmark duration (seconds)")
    parser.add_argument("--concurrency", type=int, default=10, help="Concurrent requests")
    parser.add_argument("--agents", type=int, default=1000, help="Number of test agents")

    args = parser.parse_args()

    config = BenchmarkConfig(
        layer5_url=args.url,
        duration=args.duration,
        concurrency=args.concurrency,
        num_agents=args.agents
    )

    async with Layer5Benchmarker(config) as benchmarker:
        try:
            summary = await benchmarker.run_full_benchmark()

            print("\nBenchmark Summary:")
            print(f"Overall Status: {summary['overall_status']}")
            print(f"Tests Run: {summary['total_tests']}")
            print(f"Passed: {summary['passed']}")
            print(f"Warnings: {summary['warnings']}")
            print(f"Failed: {summary['failed']}")

            if summary['overall_status'] == 'FAIL':
                print("\n❌ Benchmark FAILED - Performance targets not met")
                exit(1)
            elif summary['overall_status'] == 'WARN':
                print("\n⚠️  Benchmark PASSED with warnings - Review performance")
                exit(0)
            else:
                print("\n✅ Benchmark PASSED - All performance targets met")
                exit(0)

        except Exception as e:
            print(f"Benchmark failed: {e}")
            exit(1)

if __name__ == "__main__":
    asyncio.run(main())