#!/usr/bin/env python3
"""
Performance Monitoring Dashboard
This script provides a simple dashboard for monitoring the performance
of the code block processing system in real-time.
"""

import json
import time
import subprocess
import argparse
import sys
from datetime import datetime
from pathlib import Path

try:
    import matplotlib.pyplot as plt
    import numpy as np
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False
    print("Warning: matplotlib not available, graphical features disabled")

class PerformanceMonitor:
    def __init__(self, results_dir="performance_results"):
        self.results_dir = Path(results_dir)
        self.results_dir.mkdir(exist_ok=True)
        self.metrics_history = []
        
    def run_performance_test(self, quick=False):
        """Run a performance test and return the results"""
        try:
            cmd = ["cargo", "run", "--release", "--bin", "performance-test-runner"]
            if quick:
                cmd.append("--quick-test")
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300 if quick else 1800  # 5 min for quick, 30 min for full
            )
            
            if result.returncode == 0:
                return self.parse_performance_output(result.stdout)
            else:
                print(f"Performance test failed: {result.stderr}")
                return None
                
        except subprocess.TimeoutExpired:
            print("Performance test timed out")
            return None
        except Exception as e:
            print(f"Error running performance test: {e}")
            return None
    
    def parse_performance_output(self, output):
        """Parse performance test output and extract metrics"""
        metrics = {
            'timestamp': datetime.now().isoformat(),
            'throughput': 0.0,
            'latency_avg': 0.0,
            'latency_p95': 0.0,
            'memory_usage': 0.0,
            'cache_hit_ratio': 0.0,
            'success_rate': 0.0,
            'tests_passed': 0,
            'tests_total': 0
        }
        
        lines = output.split('\n')
        
        for line in lines:
            line = line.strip()
            
            # Extract throughput
            if 'ops/sec' in line:
                try:
                    parts = line.split()
                    for i, part in enumerate(parts):
                        if 'ops/sec' in part and i > 0:
                            metrics['throughput'] = max(metrics['throughput'], 
                                                      float(parts[i-1]))
                except (ValueError, IndexError):
                    pass
            
            # Extract latency
            if 'avg=' in line and 'ms' in line:
                try:
                    avg_part = line.split('avg=')[1].split(',')[0]
                    if 'ms' in avg_part:
                        metrics['latency_avg'] = float(avg_part.replace('ms', ''))
                except (ValueError, IndexError):
                    pass
            
            # Extract memory usage
            if 'Memory:' in line and 'MB' in line:
                try:
                    parts = line.split()
                    for part in parts:
                        if 'MB' in part:
                            metrics['memory_usage'] = max(metrics['memory_usage'],
                                                        float(part.replace('MB', '')))
                except (ValueError, IndexError):
                    pass
            
            # Extract cache hit ratio
            if 'Cache:' in line and '%' in line:
                try:
                    parts = line.split('hit_ratio=')[1].split('%')[0]
                    metrics['cache_hit_ratio'] = float(parts)
                except (ValueError, IndexError):
                    pass
            
            # Extract success rate
            if 'success' in line.lower() and '%' in line:
                try:
                    parts = line.split()
                    for part in parts:
                        if '%' in part and 'success' in line.lower():
                            metrics['success_rate'] = float(part.replace('%', ''))
                            break
                except (ValueError, IndexError):
                    pass
            
            # Extract test counts
            if 'Passing Tests:' in line:
                try:
                    metrics['tests_passed'] = int(line.split(':')[1].strip())
                except (ValueError, IndexError):
                    pass
            
            if 'Total Tests:' in line:
                try:
                    metrics['tests_total'] = int(line.split(':')[1].strip())
                except (ValueError, IndexError):
                    pass
        
        return metrics
    
    def save_metrics(self, metrics):
        """Save metrics to file"""
        if metrics:
            self.metrics_history.append(metrics)
            
            # Save to JSON file
            metrics_file = self.results_dir / "metrics_history.json"
            with open(metrics_file, 'w') as f:
                json.dump(self.metrics_history, f, indent=2)
            
            # Save latest metrics
            latest_file = self.results_dir / "latest_metrics.json"
            with open(latest_file, 'w') as f:
                json.dump(metrics, f, indent=2)
    
    def load_metrics_history(self):
        """Load metrics history from file"""
        metrics_file = self.results_dir / "metrics_history.json"
        if metrics_file.exists():
            try:
                with open(metrics_file, 'r') as f:
                    self.metrics_history = json.load(f)
            except Exception as e:
                print(f"Error loading metrics history: {e}")
                self.metrics_history = []
    
    def print_metrics(self, metrics):
        """Print metrics in a formatted way"""
        if not metrics:
            print("No metrics available")
            return
        
        print("\n" + "="*50)
        print(f"Performance Metrics - {metrics['timestamp']}")
        print("="*50)
        print(f"Throughput:        {metrics['throughput']:.1f} ops/sec")
        print(f"Avg Latency:       {metrics['latency_avg']:.1f} ms")
        print(f"P95 Latency:       {metrics['latency_p95']:.1f} ms")
        print(f"Memory Usage:      {metrics['memory_usage']:.1f} MB")
        print(f"Cache Hit Ratio:   {metrics['cache_hit_ratio']:.1f}%")
        print(f"Success Rate:      {metrics['success_rate']:.1f}%")
        print(f"Tests Passed:      {metrics['tests_passed']}/{metrics['tests_total']}")
        print("="*50)
    
    def generate_trend_analysis(self):
        """Generate trend analysis from metrics history"""
        if len(self.metrics_history) < 2:
            print("Not enough data for trend analysis")
            return
        
        print("\nTrend Analysis")
        print("-" * 30)
        
        # Calculate trends for key metrics
        recent = self.metrics_history[-5:]  # Last 5 measurements
        older = self.metrics_history[-10:-5] if len(self.metrics_history) >= 10 else self.metrics_history[:-5]
        
        if not older:
            print("Not enough historical data")
            return
        
        def calc_trend(metric_name):
            recent_avg = sum(m[metric_name] for m in recent) / len(recent)
            older_avg = sum(m[metric_name] for m in older) / len(older)
            
            if older_avg == 0:
                return 0, "N/A"
            
            change = ((recent_avg - older_avg) / older_avg) * 100
            trend = "↑" if change > 5 else "↓" if change < -5 else "→"
            return change, trend
        
        metrics_to_analyze = [
            ('throughput', 'Throughput'),
            ('latency_avg', 'Avg Latency'),
            ('memory_usage', 'Memory Usage'),
            ('cache_hit_ratio', 'Cache Hit Ratio'),
            ('success_rate', 'Success Rate')
        ]
        
        for metric_key, metric_name in metrics_to_analyze:
            change, trend = calc_trend(metric_key)
            print(f"{metric_name:15} {trend} {change:+6.1f}%")
    
    def plot_metrics(self, save_plot=False):
        """Plot metrics over time"""
        if not HAS_MATPLOTLIB:
            print("Matplotlib not available, cannot generate plots")
            return
        
        if len(self.metrics_history) < 2:
            print("Not enough data for plotting")
            return
        
        # Extract data
        timestamps = [datetime.fromisoformat(m['timestamp']) for m in self.metrics_history]
        throughput = [m['throughput'] for m in self.metrics_history]
        latency = [m['latency_avg'] for m in self.metrics_history]
        memory = [m['memory_usage'] for m in self.metrics_history]
        cache_hit = [m['cache_hit_ratio'] for m in self.metrics_history]
        
        # Create subplots
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(12, 8))
        fig.suptitle('Performance Metrics Over Time')
        
        # Throughput
        ax1.plot(timestamps, throughput, 'b-o')
        ax1.set_title('Throughput (ops/sec)')
        ax1.set_ylabel('Operations/sec')
        ax1.grid(True)
        
        # Latency
        ax2.plot(timestamps, latency, 'r-o')
        ax2.set_title('Average Latency (ms)')
        ax2.set_ylabel('Milliseconds')
        ax2.grid(True)
        
        # Memory Usage
        ax3.plot(timestamps, memory, 'g-o')
        ax3.set_title('Memory Usage (MB)')
        ax3.set_ylabel('Megabytes')
        ax3.grid(True)
        
        # Cache Hit Ratio
        ax4.plot(timestamps, cache_hit, 'm-o')
        ax4.set_title('Cache Hit Ratio (%)')
        ax4.set_ylabel('Percentage')
        ax4.grid(True)
        
        # Format x-axis
        for ax in [ax1, ax2, ax3, ax4]:
            ax.tick_params(axis='x', rotation=45)
        
        plt.tight_layout()
        
        if save_plot:
            plot_file = self.results_dir / f"performance_plot_{datetime.now().strftime('%Y%m%d_%H%M%S')}.png"
            plt.savefig(plot_file, dpi=300, bbox_inches='tight')
            print(f"Plot saved to: {plot_file}")
        else:
            plt.show()
    
    def continuous_monitoring(self, interval=300, quick_tests=True):
        """Run continuous performance monitoring"""
        print(f"Starting continuous monitoring (interval: {interval}s, quick_tests: {quick_tests})")
        print("Press Ctrl+C to stop")
        
        try:
            while True:
                print(f"\nRunning performance test at {datetime.now()}")
                metrics = self.run_performance_test(quick=quick_tests)
                
                if metrics:
                    self.save_metrics(metrics)
                    self.print_metrics(metrics)
                    
                    if len(self.metrics_history) > 1:
                        self.generate_trend_analysis()
                else:
                    print("Failed to collect metrics")
                
                print(f"Waiting {interval} seconds until next test...")
                time.sleep(interval)
                
        except KeyboardInterrupt:
            print("\nMonitoring stopped by user")
        except Exception as e:
            print(f"Monitoring error: {e}")

def main():
    parser = argparse.ArgumentParser(description="Performance Monitoring Dashboard")
    parser.add_argument("--results-dir", default="performance_results",
                       help="Directory to store results")
    parser.add_argument("--continuous", action="store_true",
                       help="Run continuous monitoring")
    parser.add_argument("--interval", type=int, default=300,
                       help="Monitoring interval in seconds (default: 300)")
    parser.add_argument("--quick", action="store_true",
                       help="Run quick tests only")
    parser.add_argument("--plot", action="store_true",
                       help="Generate performance plots")
    parser.add_argument("--save-plot", action="store_true",
                       help="Save plots to file instead of displaying")
    parser.add_argument("--trend", action="store_true",
                       help="Show trend analysis only")
    
    args = parser.parse_args()
    
    monitor = PerformanceMonitor(args.results_dir)
    monitor.load_metrics_history()
    
    if args.continuous:
        monitor.continuous_monitoring(args.interval, args.quick)
    elif args.plot:
        monitor.plot_metrics(args.save_plot)
    elif args.trend:
        monitor.generate_trend_analysis()
    else:
        # Single test run
        print("Running single performance test...")
        metrics = monitor.run_performance_test(quick=args.quick)
        
        if metrics:
            monitor.save_metrics(metrics)
            monitor.print_metrics(metrics)
            
            if len(monitor.metrics_history) > 1:
                monitor.generate_trend_analysis()
        else:
            print("Failed to collect metrics")
            sys.exit(1)

if __name__ == "__main__":
    main()