FROM python:3.7-stretch

WORKDIR /app
RUN pip install awscli

ARG VAULT_VERSION=1.0.1
RUN set -eu \
    && apt-get update \
    && apt-get install -y unzip curl \
    && curl "https://releases.hashicorp.com/vault/${VAULT_VERSION}/vault_${VAULT_VERSION}_linux_amd64.zip" -o /tmp/vault.zip \
    && unzip /tmp/vault.zip -d /usr/local/bin \
    && rm /tmp/vault.zip \
    && aws --version \
    && vault version

COPY ./ ./

CMD ["/app/configure.sh"]
