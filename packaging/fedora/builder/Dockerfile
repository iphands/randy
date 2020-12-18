FROM fedora:rawhide
RUN dnf install -y fedora-packager fedora-review cargo rust rust-packaging \
        rust-lazy_static+default-devel.noarch \
        rust-gtk+default-devel.noarch \
        rust-yaml-rust+default-devel.noarch

RUN adduser -u 1000 iphands
USER iphands

CMD ["/bin/bash", "/randy/packaging/fedora/builder/build_it.sh"]
