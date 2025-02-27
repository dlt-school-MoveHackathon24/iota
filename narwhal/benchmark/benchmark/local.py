# Copyright(C) Facebook, Inc. and its affiliates.
# Copyright (c) Mysten Labs, Inc.
# Modifications Copyright (c) 2024 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0
import subprocess
from math import ceil
from os.path import basename, splitext
from time import sleep

from benchmark.commands import CommandMaker
from benchmark.config import LocalCommittee, NodeParameters, LocalWorkerCache, BenchParameters, ConfigError
from benchmark.logs import LogParser, ParseError
from benchmark.utils import Print, BenchError, PathMaker


class LocalBench:
    BASE_PORT = 3000

    def __init__(self, bench_parameters_dict, node_parameters_dict):
        try:
            self.bench_parameters = BenchParameters(bench_parameters_dict)
            self.node_parameters = NodeParameters(node_parameters_dict)
        except ConfigError as e:
            raise BenchError('Invalid nodes or bench parameters', e)

    def __getattr__(self, attr):
        return getattr(self.bench_parameters, attr)

    def _background_run(self, command, log_file):
        name = splitext(basename(log_file))[0]
        cmd = f'{command} 2> {log_file}'
        subprocess.run(['tmux', 'new', '-d', '-s', name, cmd], check=True)

    def _kill_nodes(self):
        try:
            cmd = CommandMaker.kill().split()
            subprocess.run(cmd, stderr=subprocess.DEVNULL)
        except subprocess.SubprocessError as e:
            raise BenchError('Failed to kill testbed', e)

    def run(self, debug=False, failpoints=False, release=True):
        assert isinstance(debug, bool)
        Print.heading('Starting local benchmark')

        # Kill any previous testbed.
        self._kill_nodes()

        try:
            Print.info('Setting up testbed...')
            nodes, rate = self.nodes[0], self.rate[0]

            # Cleanup all files.
            cmd = f'{CommandMaker.clean_logs()} ; {CommandMaker.cleanup()}'
            subprocess.run([cmd], shell=True, stderr=subprocess.DEVNULL)
            sleep(0.5)  # Removing the store may take time.

            # Recompile the latest code.
            cmd = CommandMaker.compile(failpoints=failpoints, release=release)
            Print.info(f"About to run {cmd}...")
            subprocess.run(cmd, check=True, cwd=PathMaker.node_crate_path())

            # Create alias for the client and nodes binary.
            cmd = CommandMaker.alias_binaries(PathMaker.binary_path(release))
            subprocess.run([cmd], shell=True)

            # Generate configuration files.
            primary_names = []
            primary_key_files = [
                PathMaker.primary_key_file(i) for i in range(nodes)]
            for filename in primary_key_files:
                cmd = CommandMaker.generate_key(filename).split()
                subprocess.run(cmd, check=True)
                cmd_pk = CommandMaker.get_pub_key(filename).split()
                pk = subprocess.check_output(cmd_pk, encoding='utf-8').strip()
                primary_names += [pk]

            primary_network_names = []
            primary_network_key_files = [
                PathMaker.primary_network_key_file(i) for i in range(nodes)]
            for filename in primary_network_key_files:
                cmd = CommandMaker.generate_network_key(filename).split()
                subprocess.run(cmd, check=True)
                cmd_pk = CommandMaker.get_pub_key(filename).split()
                pk = subprocess.check_output(cmd_pk, encoding='utf-8').strip()
                primary_network_names += [pk]
            committee = LocalCommittee(
                primary_names, primary_network_names, self.BASE_PORT)
            committee.print(PathMaker.committee_file())

            worker_names = []
            worker_key_files = [PathMaker.worker_key_file(
                i) for i in range(self.workers*nodes)]
            for filename in worker_key_files:
                cmd = CommandMaker.generate_network_key(filename).split()
                subprocess.run(cmd, check=True)
                cmd_pk = CommandMaker.get_pub_key(filename).split()
                pk = subprocess.check_output(cmd_pk, encoding='utf-8').strip()
                worker_names += [pk]

            # 2 ports used per authority so add 2 * num authorities to base port
            worker_cache = LocalWorkerCache(
                primary_names, worker_names, self.BASE_PORT +
                (2 * len(primary_names)),
                self.workers)
            worker_cache.print(PathMaker.workers_file())

            self.node_parameters.print(PathMaker.parameters_file())

            # Run the clients (they will wait for the nodes to be ready).
            workers_addresses = worker_cache.workers_addresses(self.faults)
            rate_share = ceil(rate / worker_cache.workers())
            for i, addresses in enumerate(workers_addresses):
                for (id, address) in addresses:
                    cmd = CommandMaker.run_client(
                        address,
                        self.tx_size,
                        rate_share,
                        [x for y in workers_addresses for _, x in y]
                    )
                    log_file = PathMaker.client_log_file(i, id)
                    self._background_run(cmd, log_file)

            # Run the primaries (except the faulty ones).
            for i, address in enumerate(committee.primary_addresses(self.faults)):
                cmd = CommandMaker.run_primary(
                    PathMaker.primary_key_file(i),
                    PathMaker.primary_network_key_file(i),
                    PathMaker.worker_key_file(0),
                    PathMaker.committee_file(),
                    PathMaker.workers_file(),
                    PathMaker.db_path(i),
                    PathMaker.parameters_file(),
                    debug=debug
                )
                log_file = PathMaker.primary_log_file(i)
                self._background_run(cmd, log_file)

            # Run the workers (except the faulty ones).
            for i, addresses in enumerate(workers_addresses):
                for (id, address) in addresses:
                    cmd = CommandMaker.run_worker(
                        PathMaker.primary_key_file(i),
                        PathMaker.primary_network_key_file(i),
                        PathMaker.worker_key_file(i*self.workers + id),
                        PathMaker.committee_file(),
                        PathMaker.workers_file(),
                        PathMaker.db_path(i, id),
                        PathMaker.parameters_file(),
                        id,  # The worker's id.
                        debug=debug
                    )
                    log_file = PathMaker.worker_log_file(i, id)
                    self._background_run(cmd, log_file)

            # Wait for all transactions to be processed.
            Print.info(f'Running benchmark ({self.duration} sec)...')
            sleep(self.duration)
            self._kill_nodes()

            # Parse logs and return the parser.
            Print.info('Parsing logs...')
            return LogParser.process(PathMaker.logs_path(), faults=self.faults)

        except (subprocess.SubprocessError, ParseError) as e:
            self._kill_nodes()
            raise BenchError('Failed to run benchmark', e)
