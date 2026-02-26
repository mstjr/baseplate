FROM python:3.14-alpine

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

RUN apk add --no-cache \
    libstdc++ \
    libgcc \
    tzdata

RUN addgroup -S runner && adduser -S runner -G runner

WORKDIR /app

COPY ./sdk.py /app/sdk.py

RUN pip install \
    requests \
    pandas \
    numpy \
    python-dateutil \
    pytz \
    scipy

COPY ./bootstrap.py /app/bootstrap.py

USER runner

CMD ["python", "bootstrap.py"]