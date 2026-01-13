Name:           jico
Version:        %{?version}%{!?version:0.0.1}
Release:        1%{?dist}
Summary:        CLI helper for Jira Cloud
License:        MIT
URL:            https://example.com/jico
Source0:        %{name}-%{version}.tar.gz
BuildRequires:  rust
BuildRequires:  cargo
BuildArch:      x86_64

%description
CLI helper for Jira Cloud.

%prep
%setup -q

%build
cargo build --release

%install
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm644 packaging/jico.1 %{buildroot}%{_mandir}/man1/%{name}.1
install -Dm644 env.example %{buildroot}%{_datadir}/doc/%{name}/env.example

%files
%{_bindir}/%{name}
%{_mandir}/man1/%{name}.1*
%{_datadir}/doc/%{name}/env.example

%changelog
* Wed Dec 17 2025 jico maintainer <noreply@example.com> - %{version}-1
- Initial RPM build; includes CLI commands for create/list/view/update/transition with labels/priority/assignee support
