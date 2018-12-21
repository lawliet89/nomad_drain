FROM python:3.7-stretch

WORKDIR /app
COPY requirements.txt ./
RUN pip install -r requirements.txt

CMD ["moto_server", "iam", "-H" "0.0.0.0"]
EXPOSE 5000
