# This file is part of arclightning.
#
# arclightning is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# arclightning is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with arclightning.  If not, see <https:#www.gnu.org/licenses/>.

FROM rust:1.28 as build

# Copy over the source
COPY arclightning_backend ./arclightning_backend
# Set workdir
WORKDIR arclightning_backend
# Build
RUN cargo build
# TODO: multi stage build
CMD [ "./target/debug/arclightning_backend" ]
